from __future__ import annotations

from typing import TYPE_CHECKING

from sec_af.agents._utils import extract_harness_result
from sec_af.config import DepthProfile
from sec_af.schemas.hunt import HuntResult

from .business_logic import is_business_logic_hunter_enabled, run_business_logic_hunter

if TYPE_CHECKING:
    from .business_logic import HarnessCapable
    from sec_af.schemas.recon import ReconResult


def is_logic_hunter_enabled(depth: str | DepthProfile) -> bool:
    return is_business_logic_hunter_enabled(depth)


async def run_logic_hunter(
    app: HarnessCapable,
    repo_path: str,
    recon: ReconResult,
    depth: str | DepthProfile,
    max_files_without_signal: int = 30,
    depth_prompt: str = "",
) -> HuntResult:
    return await run_business_logic_hunter(
        app=app,
        repo_path=repo_path,
        recon_result=recon,
        depth=depth,
        max_files_without_signal=max_files_without_signal,
        depth_prompt=depth_prompt,
    )
