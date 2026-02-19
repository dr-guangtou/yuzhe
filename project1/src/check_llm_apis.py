#!/usr/bin/env python3
"""Check availability of all configured LLM API providers.

Sends a minimal "ping" prompt to each provider and reports status,
latency, and model info. Useful for diagnosing API key issues,
network problems, or provider outages before running the pipeline.

Usage:
    uv run python src/check_llm_apis.py           # Test all providers
    uv run python src/check_llm_apis.py kimi glm   # Test specific providers
    uv run python src/check_llm_apis.py --timeout 10  # Custom timeout
"""

import os
import sys
import time
import argparse
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from config import load_config, get_default_config_path
from llm_client import create_single_client

PING_PROMPT = "Reply with exactly one word: hello"


def check_api_key(provider_name, provider_config):
    """Check if the API key environment variable is set."""
    env_var = provider_config.api_key_env
    key = os.environ.get(env_var)
    if not key:
        return None, env_var
    # Mask the key for display
    masked = key[:4] + "..." + key[-4:] if len(key) > 12 else "****"
    return masked, env_var


def check_provider(provider_name, provider_config, llm_config, timeout):
    """Test a single provider with a minimal prompt.

    Returns:
        dict with keys: provider, status, detail, latency_ms, model
    """
    result = {
        "provider": provider_name,
        "status": "UNKNOWN",
        "detail": "",
        "latency_ms": None,
        "model": provider_config.default_model,
    }

    # Check API key
    masked_key, env_var = check_api_key(provider_name, provider_config)
    if masked_key is None:
        result["status"] = "NO_KEY"
        result["detail"] = f"{env_var} not set"
        return result

    # Create client
    try:
        client = create_single_client(
            provider_name=provider_name,
            provider_config=provider_config,
            model="",
            temperature=llm_config.temperature,
            max_tokens=200,
            request_timeout=float(timeout),
        )
    except ValueError as e:
        result["status"] = "INIT_FAIL"
        result["detail"] = str(e)
        return result

    # Send ping prompt
    start = time.monotonic()
    try:
        response = client.generate(
            prompt=PING_PROMPT,
            max_tokens=200,
            request_timeout=float(timeout),
        )
        elapsed_ms = int((time.monotonic() - start) * 1000)
        content = response.content or ""
        result["status"] = "OK"
        result["detail"] = content[:60] if content else "(empty response)"
        result["latency_ms"] = elapsed_ms
        result["model"] = response.model
    except Exception as e:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        result["latency_ms"] = elapsed_ms
        error_str = str(e)
        if "401" in error_str or "Unauthorized" in error_str:
            result["status"] = "AUTH_FAIL"
            result["detail"] = "Invalid API key (401)"
        elif "429" in error_str or "rate" in error_str.lower():
            result["status"] = "RATE_LIM"
            result["detail"] = "Rate limited (429)"
        elif "timeout" in error_str.lower() or "timed out" in error_str.lower():
            result["status"] = "TIMEOUT"
            result["detail"] = f"Timed out after {elapsed_ms}ms"
        elif "404" in error_str:
            result["status"] = "NOT_FOUND"
            result["detail"] = "Endpoint not found (404)"
        else:
            result["status"] = "ERROR"
            result["detail"] = error_str[:80]

    return result


def format_status(status):
    """Format status with visual indicator."""
    indicators = {
        "OK": "[PASS]",
        "NO_KEY": "[SKIP]",
        "AUTH_FAIL": "[FAIL]",
        "RATE_LIM": "[WARN]",
        "TIMEOUT": "[FAIL]",
        "NOT_FOUND": "[FAIL]",
        "INIT_FAIL": "[FAIL]",
        "ERROR": "[FAIL]",
        "UNKNOWN": "[????]",
    }
    return indicators.get(status, "[????]")


def main():
    parser = argparse.ArgumentParser(
        description="Check availability of configured LLM API providers"
    )
    parser.add_argument(
        "providers",
        nargs="*",
        help="Provider names to test (default: all configured providers)"
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="Path to config.yaml"
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=30,
        help="Request timeout in seconds (default: 30)"
    )
    args = parser.parse_args()

    # Load config
    config_path = args.config if args.config else get_default_config_path()
    try:
        config = load_config(config_path)
    except (FileNotFoundError, ValueError) as e:
        print(f"Error loading config: {e}")
        sys.exit(1)

    # Determine which providers to test
    if args.providers:
        provider_names = args.providers
        # Validate names
        for name in provider_names:
            try:
                config.get_provider(name)
            except ValueError:
                print(f"Error: provider '{name}' not found in config")
                print(f"Available: {', '.join(config.providers.keys())}")
                sys.exit(1)
    else:
        provider_names = list(config.providers.keys())

    # Header
    primary = config.llm.provider
    fallback = config.llm_fallback or []
    print("LLM API Health Check")
    print(f"Primary: {primary} | Fallback: {', '.join(fallback) if fallback else 'none'}")
    print(f"Testing {len(provider_names)} provider(s)...")
    print()

    # Test each provider
    results = []
    for name in provider_names:
        prov_config = config.get_provider(name)
        sys.stdout.write(f"  Testing {name}...")
        sys.stdout.flush()

        result = check_provider(name, prov_config, config.llm, args.timeout)
        results.append(result)

        # Inline progress
        if result["latency_ms"] is not None:
            print(f" {format_status(result['status'])} ({result['latency_ms']}ms)")
        else:
            print(f" {format_status(result['status'])}")

    # Summary table
    print()
    print(f"{'Provider':<12} {'Status':<10} {'Model':<28} {'Latency':>8}  Detail")
    print("-" * 90)

    ok_count = 0
    for r in results:
        latency = f"{r['latency_ms']}ms" if r["latency_ms"] is not None else "-"
        role = ""
        if r["provider"] == primary:
            role = " *"
        elif r["provider"] in fallback:
            role = " F"
        print(
            f"{r['provider']:<12} {r['status']:<10} {r['model']:<28} {latency:>8}  {r['detail']}"
            f"{role}"
        )
        if r["status"] == "OK":
            ok_count += 1

    print("-" * 90)
    print("  * = primary, F = fallback")
    print(f"  {ok_count}/{len(results)} providers available")

    # Exit code: 0 if primary or at least one fallback is OK
    active_names = [primary] + fallback
    active_ok = any(
        r["status"] == "OK" for r in results if r["provider"] in active_names
    )
    if not active_ok:
        print("\nWARNING: No active provider (primary + fallback) is available!")
        sys.exit(1)


if __name__ == "__main__":
    main()
