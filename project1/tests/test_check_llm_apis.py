"""Tests for check_llm_apis timeout behavior."""

from config import LLMConfig, ProviderConfig
from llm_client import LLMResponse

import check_llm_apis


def test_check_provider_passes_timeout_to_client(monkeypatch):
    """CLI timeout should be passed to both client init and generate()."""
    observed = {"create_timeout": None, "generate_timeout": None}

    class FakeClient:
        def generate(self, prompt, max_tokens=None, request_timeout=None):
            observed["generate_timeout"] = request_timeout
            return LLMResponse(content="hello", model="test-model", provider="test")

    def fake_create_single_client(
        provider_name,
        provider_config,
        model="",
        temperature=0.3,
        max_tokens=2000,
        request_timeout=120.0,
    ):
        observed["create_timeout"] = request_timeout
        return FakeClient()

    monkeypatch.setattr(check_llm_apis, "create_single_client", fake_create_single_client)
    monkeypatch.setattr(check_llm_apis, "check_api_key", lambda *_: ("****", "TEST_API_KEY"))

    provider_config = ProviderConfig(
        api_key_env="TEST_API_KEY",
        base_url="https://example.com/v1",
        default_model="test-model",
    )
    llm_config = LLMConfig(provider="test")

    result = check_llm_apis.check_provider(
        provider_name="test",
        provider_config=provider_config,
        llm_config=llm_config,
        timeout=7,
    )

    assert result["status"] == "OK"
    assert observed["create_timeout"] == 7.0
    assert observed["generate_timeout"] == 7.0
