from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class PortContext:
    source_root: Path
    tests_root: Path
    assets_root: Path
    archive_root: Path
    python_file_count: int
    test_file_count: int
    asset_file_count: int
    archive_available: bool


def build_port_context(base: Path | None = None) -> PortContext:
    root = base or Path(__file__).resolve().parent.parent
    source_root = root / 'src'
    tests_root = root / 'tests'
    assets_root = root / 'assets'
    archive_root = root / 'archive' / 'clues_code_ts_snapshot' / 'src'
    return PortContext(
        source_root=source_root,
        tests_root=tests_root,
        assets_root=assets_root,
        archive_root=archive_root,
        python_file_count=sum(1 for path in source_root.rglob('*.py') if path.is_file()),
        test_file_count=sum(1 for path in tests_root.rglob('*.py') if path.is_file()),
        asset_file_count=sum(1 for path in assets_root.rglob('*') if path.is_file()),
        archive_available=archive_root.exists(),
    )


def render_context(context: PortContext) -> str:
    return '\n'.join([
        f'Source root: {context.source_root}',
        f'Test root: {context.tests_root}',
        f'Assets root: {context.assets_root}',
        f'Archive root: {context.archive_root}',
        f'Python files: {context.python_file_count}',
        f'Test files: {context.test_file_count}',
        f'Assets: {context.asset_file_count}',
        f'Archive available: {context.archive_available}',
    ])
