"""State management for tracking runs and avoiding duplicate processing."""

import json
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Optional


@dataclass
class RunState:
    """State of the last run."""
    last_run_time: str  # ISO format datetime
    last_arxiv_date: str  # Date of papers processed (YYYY-MM-DD)
    papers_processed: int
    most_relevant_count: int
    somewhat_relevant_count: int
    could_be_interesting_count: int


class StateManager:
    """Manage run state for update mode."""

    def __init__(self, state_path: Path):
        """Initialize state manager.

        Args:
            state_path: Path to the state JSON file
        """
        self.state_path = state_path

    def load_state(self) -> Optional[RunState]:
        """Load the last run state.

        Returns:
            RunState if exists, None otherwise
        """
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
        """Save the current run state.

        Args:
            state: RunState to save
        """
        self.state_path.parent.mkdir(parents=True, exist_ok=True)

        # Write atomically
        temp_path = self.state_path.with_suffix(".tmp")
        try:
            with open(temp_path, "w", encoding="utf-8") as f:
                json.dump(asdict(state), f, indent=2)
            temp_path.rename(self.state_path)
        except Exception:
            if temp_path.exists():
                temp_path.unlink()
            raise

    def should_run(self) -> tuple[bool, str]:
        """Check if we should run based on arXiv update schedule.

        arXiv updates Monday-Friday, with new papers available around 00:00 GMT.
        Papers submitted on Friday appear Monday.
        No updates on Saturday/Sunday.

        Returns:
            Tuple of (should_run, reason_message)
        """
        now = datetime.now()
        today = now.date()
        weekday = today.weekday()  # 0=Monday, 6=Sunday

        # Check if it's a weekend
        if weekday == 5:  # Saturday
            return False, "No arXiv updates on Saturday"
        if weekday == 6:  # Sunday
            return False, "No arXiv updates on Sunday"

        # Load previous state
        state = self.load_state()

        if state is None:
            return True, "No previous run state found"

        # Parse last run date
        try:
            last_date = datetime.fromisoformat(state.last_arxiv_date).date()
        except ValueError:
            return True, "Invalid last run date in state"

        # Check if we already processed today's papers
        if last_date >= today:
            return False, f"Already processed papers for {last_date}"

        # Check if we already ran today
        try:
            last_run = datetime.fromisoformat(state.last_run_time)
            if last_run.date() == today:
                # Ran today but for different date - something might be wrong
                # Allow re-run in case of partial failure
                pass
        except ValueError:
            pass

        return True, f"New papers available since {last_date}"

    def create_state(
        self,
        papers_processed: int,
        most_relevant: int,
        somewhat_relevant: int,
        could_be_interesting: int
    ) -> RunState:
        """Create a new run state.

        Args:
            papers_processed: Total papers processed
            most_relevant: Count of most relevant papers
            somewhat_relevant: Count of somewhat relevant papers
            could_be_interesting: Count of could be interesting papers

        Returns:
            New RunState object
        """
        now = datetime.now()
        return RunState(
            last_run_time=now.isoformat(),
            last_arxiv_date=now.strftime("%Y-%m-%d"),
            papers_processed=papers_processed,
            most_relevant_count=most_relevant,
            somewhat_relevant_count=somewhat_relevant,
            could_be_interesting_count=could_be_interesting
        )


def get_default_state_path(config_path: Path = None) -> Path:
    """Get the default state file path.

    Args:
        config_path: Path to config file (determines base directory)

    Returns:
        Path to state file
    """
    if config_path:
        base_dir = config_path.parent
    else:
        base_dir = Path(".")

    return base_dir / ".state.json"
