"""Configuration loader and validation for Daily arXiv Summary."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import yaml


@dataclass
class CategoryConfig:
    """arXiv categories to monitor."""
    primary: list[str] = field(default_factory=list)
    secondary: list[str] = field(default_factory=list)

    def all_categories(self) -> list[str]:
        """Return all categories (primary + secondary)."""
        return self.primary + self.secondary


@dataclass
class TopicConfig:
    """Scientific topics of interest."""
    primary: list[str] = field(default_factory=list)
    secondary: list[str] = field(default_factory=list)
    negative: list[str] = field(default_factory=list)

    def all_topics(self) -> list[str]:
        """Return all topics (primary + secondary, excludes negative)."""
        return self.primary + self.secondary


@dataclass
class Project:
    """A project to follow."""
    name: str
    acronym: str


@dataclass
class ProviderConfig:
    """Configuration for a single LLM provider."""
    api_key_env: str
    base_url: str = ""
    default_model: str = ""
    client_type: str = "openai"  # "openai" or "gemini"
    user_agent: str = ""  # custom User-Agent (required by some providers)


@dataclass
class LLMConfig:
    """LLM provider configuration."""
    provider: str = "gemini"
    model: str = ""  # empty = use provider default
    temperature: float = 0.3
    max_tokens: int = 2000


@dataclass
class TierThresholdsConfig:
    """Thresholds for tier assignment (0-10 scale)."""
    most_relevant: float = 8.0
    somewhat_relevant: float = 6.0
    could_be_interesting: float = 3.0


@dataclass
class OutputConfig:
    """Output path configuration."""
    digest_dir: str = "arxiv_digest"
    archive_subdir: str = "archive"
    obsidian_vault: Optional[str] = None


@dataclass
class APIConfig:
    """API settings."""
    arxiv_delay_seconds: float = 3.0
    arxiv_max_results_per_category: int = 100
    llm_retry_attempts: int = 3
    llm_retry_delay_seconds: float = 5.0


@dataclass
class LocalFilterConfig:
    """Stage 1: Local embedding-based filter (MANDATORY)."""
    interest_model: str = "song_db/artifacts/interest_model.json"
    threshold: float = 0.5  # 0-1 scale (0.5 = 5.0/10)
    weights: dict[str, float] = field(default_factory=lambda: {
        "w_topic": 0.60,
        "w_global": 0.30,
        "w_category": 0.10,
    })


@dataclass
class TopicScorerConfig:
    """Stage 2 (no-LLM): Topic-embedding scorer breakpoints and thresholds."""
    primary_breakpoints: dict[float, float] = field(default_factory=lambda: {
        0.20: 0.0, 0.35: 3.0, 0.45: 5.0, 0.55: 8.0, 0.70: 10.0,
    })
    secondary_breakpoints: dict[float, float] = field(default_factory=lambda: {
        0.20: 0.0, 0.30: 2.0, 0.40: 4.0, 0.50: 6.0, 0.65: 8.0,
    })
    match_threshold: float = 0.35
    multi_match_bonus: float = 0.5
    negative_penalty: float = 3.0


@dataclass
class LLMScoringConfig:
    """Stage 2: LLM-based scoring (OPTIONAL)."""
    enabled: bool = False  # Enable via --use-llm-scoring
    summary_tiers: list[str] = field(default_factory=lambda: ["most_relevant", "somewhat_relevant"])
    tier_thresholds: TierThresholdsConfig = field(default_factory=TierThresholdsConfig)


@dataclass
class SummaryConfig:
    """Stage 3: Summary generation (OPTIONAL)."""
    enabled: bool = True  # Disable via --no-summary
    fallback_to_abstract: bool = True


@dataclass
class Config:
    """Main configuration container."""
    category: CategoryConfig
    topics: TopicConfig
    projects: list[Project]
    llm: LLMConfig
    output: OutputConfig
    api: APIConfig
    local_filter: LocalFilterConfig
    topic_scorer: TopicScorerConfig
    llm_scoring: LLMScoringConfig
    summary: SummaryConfig
    providers: dict[str, ProviderConfig] = field(default_factory=dict)
    llm_fallback: list[str] = field(default_factory=list)
    config_path: Optional[Path] = None

    def get_provider(self, name: str) -> ProviderConfig:
        """Look up a provider by name.

        Args:
            name: Provider name (must exist in providers dict).

        Raises:
            ValueError: If provider name is not registered.
        """
        if name not in self.providers:
            raise ValueError(
                f"Unknown provider '{name}'. "
                f"Available: {list(self.providers.keys())}"
            )
        return self.providers[name]

    def get_project_keywords(self) -> list[str]:
        """Get all project names and acronyms for title matching."""
        keywords = []
        for project in self.projects:
            keywords.append(project.name.lower())
            keywords.append(project.acronym.lower())
        return keywords

    def get_digest_path(self, year: int) -> Path:
        """Get the path to the digest archive for a given year."""
        if self.config_path:
            base = self.config_path.parent
        else:
            base = Path(".")
        return base / self.output.digest_dir / self.output.archive_subdir / str(year)


def load_config(config_path: Path) -> Config:
    """Load and validate configuration from YAML file.

    Args:
        config_path: Path to the config.yaml file.

    Returns:
        Validated Config object.

    Raises:
        FileNotFoundError: If config file doesn't exist.
        ValueError: If config is invalid.
    """
    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")

    with open(config_path, "r", encoding="utf-8") as f:
        raw = yaml.safe_load(f)

    if raw is None:
        raise ValueError("Config file is empty")

    # Parse category config
    cat_raw = raw.get("category", {})
    category = CategoryConfig(
        primary=cat_raw.get("primary", []),
        secondary=cat_raw.get("secondary", [])
    )

    # Parse topics config
    topics_raw = raw.get("topics", {})
    topics = TopicConfig(
        primary=topics_raw.get("primary", []),
        secondary=topics_raw.get("secondary", []),
        negative=topics_raw.get("negative", []),
    )

    # Parse projects
    projects_raw = raw.get("projects", [])
    projects = []
    for p in projects_raw:
        if isinstance(p, dict):
            projects.append(Project(
                name=p.get("name", ""),
                acronym=p.get("acronym", "")
            ))
        elif isinstance(p, str):
            # Handle old format: "Name (ACRONYM)"
            projects.append(Project(name=p, acronym=""))

    # Parse providers
    providers_raw = raw.get("providers", {})
    providers = {}
    for name, prov_data in providers_raw.items():
        if not isinstance(prov_data, dict):
            continue
        providers[name] = ProviderConfig(
            api_key_env=prov_data.get("api_key_env", ""),
            base_url=prov_data.get("base_url", ""),
            default_model=prov_data.get("default_model", ""),
            client_type=prov_data.get("client_type", "openai"),
            user_agent=prov_data.get("user_agent", ""),
        )

    # Parse LLM config
    llm_raw = raw.get("llm", {})
    llm = LLMConfig(
        provider=llm_raw.get("provider", "gemini"),
        model=llm_raw.get("model", ""),
        temperature=llm_raw.get("temperature", 0.3),
        max_tokens=llm_raw.get("max_tokens", 2000)
    )

    # Parse tier thresholds (used in LLM scoring)
    tier_raw = raw.get("llm_scoring", {}).get("tier_thresholds", {})
    tier_thresholds = TierThresholdsConfig(
        most_relevant=tier_raw.get("most_relevant", 8.0),
        somewhat_relevant=tier_raw.get("somewhat_relevant", 6.0),
        could_be_interesting=tier_raw.get("could_be_interesting", 3.0)
    )

    # Parse output config
    output_raw = raw.get("output", {})
    output = OutputConfig(
        digest_dir=output_raw.get("digest_dir", "arxiv_digest"),
        archive_subdir=output_raw.get("archive_subdir", "archive"),
        obsidian_vault=output_raw.get("obsidian_vault")
    )

    # Parse API config
    api_raw = raw.get("api", {})
    api = APIConfig(
        arxiv_delay_seconds=api_raw.get("arxiv_delay_seconds", 3.0),
        arxiv_max_results_per_category=api_raw.get("arxiv_max_results_per_category", 100),
        llm_retry_attempts=api_raw.get("llm_retry_attempts", 3),
        llm_retry_delay_seconds=api_raw.get("llm_retry_delay_seconds", 5.0)
    )

    # Parse local filter config (Stage 1 - MANDATORY)
    lf_raw = raw.get("local_filter", {})
    local_filter = LocalFilterConfig(
        interest_model=lf_raw.get("interest_model", "song_db/artifacts/interest_model.json"),
        threshold=lf_raw.get("threshold", 0.5),
        weights=lf_raw.get("weights", {"w_topic": 0.60, "w_global": 0.30, "w_category": 0.10}),
    )

    # Parse topic scorer config (Stage 2 no-LLM path)
    ts_raw = raw.get("topic_scorer", {})
    topic_scorer = TopicScorerConfig(
        primary_breakpoints={float(k): float(v) for k, v in ts_raw.get("primary_breakpoints", {0.20: 0.0, 0.35: 3.0, 0.45: 5.0, 0.55: 8.0, 0.70: 10.0}).items()},
        secondary_breakpoints={float(k): float(v) for k, v in ts_raw.get("secondary_breakpoints", {0.20: 0.0, 0.30: 2.0, 0.40: 4.0, 0.50: 6.0, 0.65: 8.0}).items()},
        match_threshold=ts_raw.get("match_threshold", 0.35),
        multi_match_bonus=ts_raw.get("multi_match_bonus", 0.5),
        negative_penalty=ts_raw.get("negative_penalty", 3.0),
    )

    # Parse LLM scoring config (Stage 2 - OPTIONAL)
    llm_scoring_raw = raw.get("llm_scoring", {})
    summary_tiers = llm_scoring_raw.get("summary_tiers")
    if summary_tiers is None:
        summary_tiers = llm_scoring_raw.get("keep_tiers", ["most_relevant", "somewhat_relevant"])
    llm_scoring = LLMScoringConfig(
        enabled=llm_scoring_raw.get("enabled", False),
        summary_tiers=summary_tiers,
        tier_thresholds=tier_thresholds,
    )

    # Parse summary config (Stage 3 - OPTIONAL)
    summary_raw = raw.get("summary", {})
    summary = SummaryConfig(
        enabled=summary_raw.get("enabled", True),
        fallback_to_abstract=summary_raw.get("fallback_to_abstract", True),
    )

    # Parse LLM fallback list
    llm_fallback = raw.get("llm_fallback", [])

    # Validate required fields
    if not category.primary:
        raise ValueError("At least one primary category is required")
    if not topics.primary:
        raise ValueError("At least one primary topic is required")

    # Validate provider references
    if providers:
        if llm.provider and llm.provider not in providers:
            raise ValueError(
                f"llm.provider '{llm.provider}' not found in providers. "
                f"Available: {list(providers.keys())}"
            )
        for fb_name in llm_fallback:
            if fb_name not in providers:
                raise ValueError(
                    f"Fallback provider '{fb_name}' not found in providers. "
                    f"Available: {list(providers.keys())}"
                )

    return Config(
        category=category,
        topics=topics,
        projects=projects,
        llm=llm,
        output=output,
        api=api,
        local_filter=local_filter,
        topic_scorer=topic_scorer,
        llm_scoring=llm_scoring,
        summary=summary,
        providers=providers,
        llm_fallback=llm_fallback,
        config_path=config_path.resolve()
    )


def get_default_config_path() -> Path:
    """Get the default config path relative to project root."""
    # Assume we're running from project1/src/
    return Path(__file__).parent.parent / "config.yaml"
