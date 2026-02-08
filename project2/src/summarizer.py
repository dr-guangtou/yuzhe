"""LLM summary generation and translation for publications."""

import logging
from pathlib import Path

import yaml

from llm_client import LLMClient, LLMResponse, retry_with_backoff
from orcid_fetcher import Publication

logger = logging.getLogger(__name__)


class Summarizer:
    """Generates bilingual summaries for publications via LLM."""

    def __init__(
        self,
        llm_client: LLMClient,
        prompts_dir: Path,
        output_dir: Path,
        model_name: str,
        retry_attempts: int = 3,
        retry_delay: float = 5.0,
    ):
        self.llm_client = llm_client
        self.prompts_dir = prompts_dir
        self.output_dir = output_dir
        self.model_name = model_name
        self.retry_attempts = retry_attempts
        self.retry_delay = retry_delay

        # Load prompt templates
        self._short_template = self._load_prompt("summary_short.md")
        self._long_template = self._load_prompt("summary_long.md")
        self._translate_template = self._load_prompt("translate.md")

    def _load_prompt(self, filename: str) -> str:
        path = self.prompts_dir / filename
        if not path.exists():
            raise FileNotFoundError(f"Prompt template not found: {path}")
        return path.read_text(encoding="utf-8")

    def _format_authors(self, pub: Publication) -> str:
        if not pub.authors:
            return "Unknown"
        if len(pub.authors) <= 3:
            return ", ".join(pub.authors)
        return f"{pub.authors[0]} et al. ({len(pub.authors)} authors)"

    def _call_llm(self, prompt: str) -> str:
        """Call LLM with retry logic."""
        response = retry_with_backoff(
            lambda: self.llm_client.generate(prompt=prompt),
            max_attempts=self.retry_attempts,
            initial_delay=self.retry_delay,
        )
        return response.content

    def generate_summary(self, pub: Publication, summary_type: str) -> str:
        """Generate a summary for a publication.

        Args:
            pub: Publication to summarize.
            summary_type: "short" or "long".
        """
        if summary_type == "short":
            template = self._short_template
        elif summary_type == "long":
            template = self._long_template
        else:
            raise ValueError(f"Unknown summary type: {summary_type}")

        prompt = template.format(
            title=pub.title,
            authors=self._format_authors(pub),
            journal=pub.journal or "Preprint",
            year=pub.year,
            abstract=pub.abstract,
        )

        return self._call_llm(prompt)

    def translate_summary(self, english_summary: str) -> str:
        """Translate an English summary to Chinese."""
        prompt = self._translate_template.format(
            english_summary=english_summary,
        )
        return self._call_llm(prompt)

    def process_publication(self, pub: Publication) -> dict[str, str]:
        """Generate all 4 summaries for a publication.

        Returns:
            {"short_eng": ..., "long_eng": ..., "short_chn": ..., "long_chn": ...}
        """
        if not pub.abstract:
            logger.warning("Skipping %s: no abstract", pub.title[:50])
            return {}

        logger.info("Generating summaries for: %s", pub.title[:60])

        short_eng = self.generate_summary(pub, "short")
        logger.info("  Short summary generated")

        long_eng = self.generate_summary(pub, "long")
        logger.info("  Long summary generated")

        short_chn = self.translate_summary(short_eng)
        logger.info("  Short Chinese translation generated")

        long_chn = self.translate_summary(long_eng)
        logger.info("  Long Chinese translation generated")

        return {
            "short_eng": short_eng,
            "long_eng": long_eng,
            "short_chn": short_chn,
            "long_chn": long_chn,
        }

    def save_summaries(self, all_summaries: dict[str, dict[str, str]]) -> None:
        """Save summaries to YAML files under summary/{model_name}/.

        Args:
            all_summaries: {doi_or_title: {short_eng, long_eng, short_chn, long_chn}}
        """
        model_dir = self.output_dir / "summary" / self.model_name
        model_dir.mkdir(parents=True, exist_ok=True)

        summary_types = {
            "summary_short_eng": "short_eng",
            "summary_long_eng": "long_eng",
            "summary_short_chn": "short_chn",
            "summary_long_chn": "long_chn",
        }

        for filename, key in summary_types.items():
            data = {}
            for pub_key, summaries in all_summaries.items():
                if key in summaries:
                    data[pub_key] = summaries[key]

            if data:
                path = model_dir / f"{filename}.yaml"
                with open(path, "w", encoding="utf-8") as f:
                    yaml.dump(
                        data, f,
                        default_flow_style=False,
                        allow_unicode=True,
                        sort_keys=False,
                    )
                logger.info("Saved %s (%d entries)", path.name, len(data))
