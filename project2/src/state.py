"""Run state tracking for the publication record pipeline."""

import json
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Optional


@dataclass
class RunState:
    """State of the last run."""
    last_run_time: str
    publications_count: int
    summaries_generated: int


class StateManager:
    """Manage run state to detect when updates are needed."""

    def __init__(self, state_path: Path):
        self.state_path = state_path

    def load_state(self) -> Optional[RunState]:
        if not self.state_path.exists():
            return None
        try:
            with open(self.state_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            return RunState(**data)
        except (json.JSONDecodeError, TypeError, KeyError) as e:
            print(f"Warning: Could not parse state file: {e}")
            return None

    def save_state(self, state: RunState) -> None:
        self.state_path.parent.mkdir(parents=True, exist_ok=True)
        temp_path = self.state_path.with_suffix(".tmp")
        try:
            with open(temp_path, "w", encoding="utf-8") as f:
                json.dump(asdict(state), f, indent=2)
            temp_path.rename(self.state_path)
        except Exception:
            if temp_path.exists():
                temp_path.unlink()
            raise

    def create_state(
        self, publications_count: int, summaries_generated: int
    ) -> RunState:
        return RunState(
            last_run_time=datetime.now().isoformat(),
            publications_count=publications_count,
            summaries_generated=summaries_generated,
        )
