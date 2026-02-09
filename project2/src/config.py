"""Configuration loader for Personal Publication Record."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import yaml


@dataclass
class ProviderConfig:
    """Configuration for a single LLM provider."""
    api_key_env: str
    base_url: str = ""
    default_model: str = ""
    client_type: str = "openai"


@dataclass
class LLMConfig:
    """LLM provider configuration."""
    provider: str = "kimi"
    model: str = ""
    temperature: float = 0.3
    max_tokens: int = 2000


@dataclass
class UserConfig:
    """User settings from config.yaml + user.yaml."""
    last_name: str = ""
    first_name: str = ""
    chinese_name: str = ""
    email: str = ""
    affiliation: str = ""
    orcid: str = ""
    google_scholar_id: str = ""
    year_cutoff: int = 2021


@dataclass
class OutputConfig:
    """Output path configuration."""
    record_dir: str = "publication_record"
    portfolio_subdir: str = "portfolio"


@dataclass
class APIConfig:
    """API settings."""
    orcid_timeout: int = 30
    ads_api_key_env: str = "ADS_API_TOKEN"
    ads_timeout: int = 30
    ads_rate_limit_delay: float = 1.0
    llm_retry_attempts: int = 3
    llm_retry_delay_seconds: float = 5.0


@dataclass
class Config:
    """Main configuration container."""
    user: UserConfig
    llm: LLMConfig
    output: OutputConfig
    api: APIConfig
    providers: dict[str, ProviderConfig] = field(default_factory=dict)
    llm_fallback: list[str] = field(default_factory=list)
    config_path: Optional[Path] = None

    def get_provider(self, name: str) -> ProviderConfig:
        if name not in self.providers:
            raise ValueError(
                f"Unknown provider '{name}'. "
                f"Available: {list(self.providers.keys())}"
            )
        return self.providers[name]

    def get_record_path(self) -> Path:
        if self.config_path:
            base = self.config_path.parent
        else:
            base = Path(".")
        return base / self.output.record_dir

    def get_portfolio_path(self) -> Path:
        return self.get_record_path() / self.output.portfolio_subdir


def load_config(config_path: Path) -> Config:
    """Load and validate configuration from YAML file."""
    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")

    with open(config_path, "r", encoding="utf-8") as f:
        raw = yaml.safe_load(f)

    if raw is None:
        raise ValueError("Config file is empty")

    # Load user.yaml from same directory
    user_yaml_path = config_path.parent / "user.yaml"
    user_data = {}
    if user_yaml_path.exists():
        with open(user_yaml_path, "r", encoding="utf-8") as f:
            user_raw = yaml.safe_load(f)
        if user_raw and "user" in user_raw:
            user_data = user_raw["user"]

    # Merge user config: user.yaml provides identity, config.yaml provides settings
    config_user = raw.get("user", {})
    user = UserConfig(
        last_name=user_data.get("last_name", ""),
        first_name=user_data.get("first_name", ""),
        chinese_name=user_data.get("chinese_name", ""),
        email=user_data.get("email", ""),
        affiliation=user_data.get("affiliation", ""),
        orcid=config_user.get("orcid", user_data.get("orcid", "")),
        google_scholar_id=user_data.get("google_scholar_id", ""),
        year_cutoff=config_user.get("year_cutoff", 2021),
    )

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
        )

    # Parse LLM config
    llm_raw = raw.get("llm", {})
    llm = LLMConfig(
        provider=llm_raw.get("provider", "kimi"),
        model=llm_raw.get("model", ""),
        temperature=llm_raw.get("temperature", 0.3),
        max_tokens=llm_raw.get("max_tokens", 2000),
    )

    # Parse output config
    output_raw = raw.get("output", {})
    output = OutputConfig(
        record_dir=output_raw.get("record_dir", "publication_record"),
        portfolio_subdir=output_raw.get("portfolio_subdir", "portfolio"),
    )

    # Parse API config
    api_raw = raw.get("api", {})
    api = APIConfig(
        orcid_timeout=api_raw.get("orcid_timeout", 30),
        ads_api_key_env=api_raw.get("ads_api_key_env", "ADS_API_TOKEN"),
        ads_timeout=api_raw.get("ads_timeout", 30),
        ads_rate_limit_delay=api_raw.get("ads_rate_limit_delay", 1.0),
        llm_retry_attempts=api_raw.get("llm_retry_attempts", 3),
        llm_retry_delay_seconds=api_raw.get("llm_retry_delay_seconds", 5.0),
    )

    # Parse fallback list
    llm_fallback = raw.get("llm_fallback", [])

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
        user=user,
        llm=llm,
        output=output,
        api=api,
        providers=providers,
        llm_fallback=llm_fallback,
        config_path=config_path.resolve(),
    )


def get_default_config_path() -> Path:
    return Path(__file__).parent.parent / "config.yaml"
