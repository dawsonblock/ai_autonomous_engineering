from __future__ import annotations

import os
import re
from typing import Any, List, Optional

from .models import StackFrameRef


class StacktraceLinker:
    FRAME_RE = re.compile(r'File "([^"]+)", line (\d+), in ([^\s]+)')

    def __init__(self, symbol_store: Any = None):
        self.symbol_store = symbol_store

    def parse(self, stacktrace: str, repo_root: str) -> List[StackFrameRef]:
        frames: List[StackFrameRef] = []

        for match in self.FRAME_RE.finditer(stacktrace or ""):
            file_path = match.group(1)
            rel_path = os.path.relpath(file_path, repo_root) if os.path.isabs(file_path) else file_path
            in_project = not rel_path.startswith("..") and not rel_path.startswith("/")
            frames.append(
                StackFrameRef(
                    file_path=rel_path,
                    line_number=int(match.group(2)),
                    function_name=match.group(3),
                    in_project=in_project,
                )
            )

        project_frames = [f for f in frames if f.in_project]
        
        weights = [1.0, 0.8, 0.6]
        project_frames.reverse()
        for i, frame in enumerate(project_frames):
            frame.weight = weights[i] if i < len(weights) else 0.4
            
            if self.symbol_store and getattr(self.symbol_store, 'enabled', False):
                frame.resolved_symbol_id = self._resolve_symbol(frame)
                
        project_frames.reverse()
        return project_frames

    def _resolve_symbol(self, frame: StackFrameRef) -> Optional[str]:
        if hasattr(self.symbol_store, 'database') and self.symbol_store.database:
            try:
                with self.symbol_store.database.connection() as conn:
                    with conn.cursor() as cur:
                        cur.execute(
                            "SELECT symbol_id FROM aae_symbols WHERE file_path = %s AND name = %s LIMIT 1",
                            (frame.file_path, frame.function_name)
                        )
                        result = cur.fetchone()
                        if result:
                            return result[0]
                        
                        cur.execute(
                            "SELECT symbol_id FROM aae_symbols WHERE name = %s LIMIT 1",
                            (frame.function_name,)
                        )
                        result = cur.fetchone()
                        if result:
                            return result[0]
            except Exception:
                pass
        return None
