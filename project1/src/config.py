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

    def all_topics(self) -> list[str]:
        """Return all topics (primary + secondary)."""
        return self.primary + self.secondary


@dataclass
class Project:
    """A project to follow."""
    name: str
    acronym: str


@dataclass
class LLMConfig:
    """LLM provider configuration."""
    provider: str = "gemini"
    model: str = "gemini-2.0-flash"
    api_key_env: str = "GEMINI_API_KEY"
    temperature: float = 0.3
    max_tokens: int = 2000


@dataclass
class ScoringConfig:
    """Scoring thresholds for tier assignment."""
    most_relevant_threshold: float = 7.0
    somewhat_relevant_threshold: float = 5.0
    could_be_interesting_threshold: float = 3.0


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
class Config:
    """Main configuration container."""
    category: CategoryConfig
    topics: TopicConfig
    projects: list[Project]
    llm: LLMConfig
    scoring: ScoringConfig
    output: OutputConfig
    api: APIConfig
    llm_fallback: list[str] = field(default_factory=list)
    config_path: Optional[Path] = None

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
        secondary=topics_raw.get("secondary", [])
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

    # Parse LLM config
    llm_raw = raw.get("llm", {})
    llm = LLMConfig(
        provider=llm_raw.get("provider", "gemini"),
        model=llm_raw.get("model", "gemini-2.0-flash"),
        api_key_env=llm_raw.get("api_key_env", "GEMINI_API_KEY"),
        temperature=llm_raw.get("temperature", 0.3),
        max_tokens=llm_raw.get("max_tokens", 2000)
    )

    # Parse scoring config
    scoring_raw = raw.get("scoring", {})
    scoring = ScoringConfig(
        most_relevant_threshold=scoring_raw.get("most_relevant_threshold", 7.0),
        somewhat_relevant_threshold=scoring_raw.get("somewhat_relevant_threshold", 5.0),
        could_be_interesting_threshold=scoring_raw.get("could_be_interesting_threshold", 3.0)
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

    # Parse LLM fallback list
    llm_fallback = raw.get("llm_fallback", [])

    # Validate required fields
    if not category.primary:
        raise ValueError("At least one primary category is required")
    if not topics.primary:
        raise ValueError("At least one primary topic is required")

    return Config(
        category=category,
        topics=topics,
        projects=projects,
        llm=llm,
        scoring=scoring,
        output=output,
        api=api,
        llm_fallback=llm_fallback,
        config_path=config_path.resolve()
    )


def get_default_config_path() -> Path:
    """Get the default config path relative to project root."""
    # Assume we're running from project1/src/
    return Path(__file__).parent.parent / "config.yaml"
