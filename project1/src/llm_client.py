"""LLM client abstraction with multiple providers and fallback support."""

import json
import os
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional

from config import LLMConfig, ProviderConfig, Config


@dataclass
class LLMResponse:
    """Response from an LLM."""
    content: str
    model: str
    provider: str


class LLMClient(ABC):
    """Abstract base class for LLM clients."""

    @abstractmethod
    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None
    ) -> LLMResponse:
        """Generate a response from the LLM."""
        pass


class OpenAICompatibleClient(LLMClient):
    """Client for OpenAI-compatible APIs (OpenAI, KIMI/Moonshot, etc.)."""

    def __init__(
        self,
        api_key: str,
        model: str,
        base_url: str,
        provider_name: str,
        temperature: float = 0.3,
        max_tokens: int = 2000,
        user_agent: str = "",
    ):
        self.api_key = api_key
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.provider_name = provider_name
        self.default_temperature = temperature
        self.default_max_tokens = max_tokens
        self.user_agent = user_agent

    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None
    ) -> LLMResponse:
        """Generate using OpenAI-compatible API."""
        import urllib.request

        temp = temperature if temperature is not None else self.default_temperature
        tokens = max_tokens if max_tokens is not None else self.default_max_tokens

        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        payload = {
            "model": self.model,
            "messages": messages,
            "temperature": temp,
            "max_tokens": tokens
        }

        url = f"{self.base_url}/chat/completions"
        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {self.api_key}",
        }
        if self.user_agent:
            headers["User-Agent"] = self.user_agent
        req = urllib.request.Request(
            url,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST"
        )

        try:
            with urllib.request.urlopen(req, timeout=120) as response:
                result = json.loads(response.read().decode("utf-8"))
        except Exception as e:
            raise RuntimeError(f"{self.provider_name} API request failed: {e}")

        try:
            content = result["choices"][0]["message"]["content"]
        except (KeyError, IndexError) as e:
            raise RuntimeError(f"Failed to parse {self.provider_name} response: {e}")

        if content is None:
            raise RuntimeError(f"{self.provider_name} returned null content")

        return LLMResponse(
            content=content.strip(),
            model=self.model,
            provider=self.provider_name
        )


class GeminiAPIClient(LLMClient):
    """Gemini client using the Google AI API."""

    def __init__(
        self,
        api_key: str,
        model: str,
        temperature: float = 0.3,
        max_tokens: int = 2000
    ):
        self.api_key = api_key
        self.model = model
        self.default_temperature = temperature
        self.default_max_tokens = max_tokens

    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None
    ) -> LLMResponse:
        """Generate using Gemini API."""
        import urllib.request

        temp = temperature if temperature is not None else self.default_temperature
        tokens = max_tokens if max_tokens is not None else self.default_max_tokens

        url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.model}:generateContent?key={self.api_key}"

        contents = []
        if system_prompt:
            contents.append({
                "role": "user",
                "parts": [{"text": f"System: {system_prompt}"}]
            })
            contents.append({
                "role": "model",
                "parts": [{"text": "Understood. I will follow these instructions."}]
            })

        contents.append({
            "role": "user",
            "parts": [{"text": prompt}]
        })

        payload = {
            "contents": contents,
            "generationConfig": {
                "temperature": temp,
                "maxOutputTokens": tokens
            }
        }

        req = urllib.request.Request(
            url,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST"
        )

        try:
            with urllib.request.urlopen(req, timeout=120) as response:
                result = json.loads(response.read().decode("utf-8"))
        except Exception as e:
            raise RuntimeError(f"Gemini API request failed: {e}")

        try:
            content = result["candidates"][0]["content"]["parts"][0]["text"]
        except (KeyError, IndexError) as e:
            raise RuntimeError(f"Failed to parse Gemini response: {e}")

        if content is None:
            raise RuntimeError("Gemini returned null content")

        return LLMResponse(
            content=content.strip(),
            model=self.model,
            provider="gemini"
        )


class MockLLMClient(LLMClient):
    """Mock LLM client for testing."""

    def __init__(self, config: LLMConfig = None):
        self.config = config

    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None
    ) -> LLMResponse:
        """Return a mock response for testing."""
        if "score" in prompt.lower() and "relevance" in prompt.lower():
            return LLMResponse(
                content='{"score": 6.0, "matched_topics": ["galaxy formation"], "reasoning": "Mock scoring response"}',
                model="mock",
                provider="mock"
            )

        if "summary" in prompt.lower() or "summarize" in prompt.lower():
            return LLMResponse(
                content="This is a mock summary of the paper for testing purposes.",
                model="mock",
                provider="mock"
            )

        return LLMResponse(
            content="Mock response",
            model="mock",
            provider="mock"
        )


class FallbackLLMClient(LLMClient):
    """LLM client that tries multiple providers in sequence."""

    def __init__(self, clients: list[LLMClient]):
        """Initialize with ordered list of clients to try.

        Args:
            clients: List of LLM clients, tried in order
        """
        if not clients:
            raise ValueError("At least one client is required")
        self.clients = clients
        self.current_client_index = 0

    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None
    ) -> LLMResponse:
        """Try each client in sequence until one succeeds."""
        errors = []

        for i, client in enumerate(self.clients):
            try:
                response = client.generate(
                    prompt=prompt,
                    system_prompt=system_prompt,
                    temperature=temperature,
                    max_tokens=max_tokens
                )
                # Remember which client worked
                self.current_client_index = i
                return response
            except Exception as e:
                errors.append(f"{type(client).__name__}: {e}")
                continue

        # All clients failed
        raise RuntimeError(f"All LLM clients failed: {'; '.join(errors)}")


def create_single_client(
    provider_name: str,
    provider_config: ProviderConfig,
    model: str = "",
    temperature: float = 0.3,
    max_tokens: int = 2000
) -> LLMClient:
    """Create a single LLM client from a ProviderConfig.

    Args:
        provider_name: Display name for the provider.
        provider_config: Provider configuration from config.yaml.
        model: Model override (empty = use provider default).
        temperature: Default temperature.
        max_tokens: Default max tokens.

    Returns:
        LLMClient instance.

    Raises:
        ValueError: If API key is missing from environment.
    """
    api_key = os.environ.get(provider_config.api_key_env)
    if not api_key:
        raise ValueError(
            f"API key not found: {provider_config.api_key_env}"
        )

    model_name = model or provider_config.default_model

    if provider_config.client_type == "gemini":
        return GeminiAPIClient(
            api_key=api_key,
            model=model_name,
            temperature=temperature,
            max_tokens=max_tokens,
        )
    else:
        return OpenAICompatibleClient(
            api_key=api_key,
            model=model_name,
            base_url=provider_config.base_url,
            provider_name=provider_name,
            temperature=temperature,
            max_tokens=max_tokens,
            user_agent=provider_config.user_agent,
        )


def create_fallback_client(config: Config) -> LLMClient:
    """Create a fallback client from the full Config.

    Uses config.llm_fallback for provider order, falling back to
    the single primary provider if no fallback list is defined.

    Args:
        config: Full application config with providers registry.

    Returns:
        FallbackLLMClient or single client if only one succeeds.

    Raises:
        ValueError: If no clients could be initialized.
    """
    primary = config.llm.provider
    fallback = config.llm_fallback if config.llm_fallback else []
    provider_names = [primary] + [name for name in fallback if name != primary]

    clients = []
    for name in provider_names:
        try:
            prov = config.get_provider(name)
            # Use model override only for the primary provider
            model = config.llm.model if name == config.llm.provider else ""
            client = create_single_client(
                provider_name=name,
                provider_config=prov,
                model=model,
                temperature=config.llm.temperature,
                max_tokens=config.llm.max_tokens,
            )
            clients.append(client)
            print(f"  Initialized {name} client")
        except ValueError as e:
            print(f"  Skipping {name}: {e}")
            continue

    if not clients:
        raise ValueError("No LLM clients could be initialized")

    if len(clients) == 1:
        return clients[0]

    return FallbackLLMClient(clients)


def retry_with_backoff(
    func,
    max_attempts: int = 3,
    initial_delay: float = 5.0,
    backoff_factor: float = 2.0
):
    """Retry a function with exponential backoff."""
    delay = initial_delay
    last_exception = None

    for attempt in range(max_attempts):
        try:
            return func()
        except Exception as e:
            last_exception = e
            if attempt < max_attempts - 1:
                print(f"Attempt {attempt + 1} failed: {e}. Retrying in {delay}s...")
                time.sleep(delay)
                delay *= backoff_factor

    raise last_exception
