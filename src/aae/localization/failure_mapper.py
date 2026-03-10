from __future__ import annotations

import re
from typing import List

from .models import FailureSignal


class FailureMapper:
    def parse_pytest_output(self, raw_output: str) -> List[FailureSignal]:
        failures: List[FailureSignal] = []

        # Split output into blocks delimited by test headers
        blocks = re.split(r"(?:^|\n)(?=_{10,}\s+[^\s]+\s+_{10,})", raw_output)
        
        for block in blocks:
            header_match = re.search(r"_{10,}\s+([^\s]+)\s+_{10,}", block)
            if header_match:
                test_name = header_match.group(1)
                file_path = None
                line_number = None
                exception_type = None
                
                # Check for "E   ExceptionType: Message"
                e_match = re.search(r"^E\s+([A-Za-z_][A-Za-z0-9_]*)(?::|\s)", block, re.MULTILINE)
                if e_match:
                    exception_type = e_match.group(1)
                
                # Check for "FAILED file.py:line: ExceptionType"
                f_match = re.search(r"FAILED\s+([^\s:]+\.py):(\d+):\s*([A-Za-z_][A-Za-z0-9_]*)?", block)
                if f_match:
                    file_path = file_path or f_match.group(1)
                    line_number = line_number or int(f_match.group(2))
                    exception_type = exception_type or f_match.group(3)
                    
                # Check for `File "...", line ...`
                trace_match = re.search(r'File "([^"]+)", line (\d+)', block)
                if trace_match:
                    file_path = file_path or trace_match.group(1)
                    line_number = line_number or int(trace_match.group(2))
                    
                # general fallback `file.py:line:`
                match = re.search(r"([^\s:]+\.py):(\d+):", block)
                if match:
                    file_path = file_path or match.group(1)
                    line_number = line_number or int(match.group(2))

                failures.append(
                    FailureSignal(
                        test_name=test_name,
                        file_path=file_path,
                        line_number=line_number,
                        exception_type=exception_type,
                        stacktrace=block.strip(),
                        raw_output=raw_output,
                    )
                )

        if not failures:
            # Fallback to scanning for "FAILED " lines anywhere (e.g. from short test summary or very noisy logs)
            for line in raw_output.splitlines():
                if line.startswith("FAILED "):
                    test_name = "unknown"
                    file_path = None
                    line_number = None
                    exception_type = None
                    
                    # FAILED tests/test_auth.py::test_auth_invalid_token - AuthenticationError: Invalid token
                    parts = line.split("FAILED ", 1)[1]
                    summary_match = re.search(r"([^\s:]+\.py)(?:::([^\s]+))?(?:\s+-\s+([^\s:]+))?", parts)
                    if summary_match:
                        file_path = summary_match.group(1)
                        if summary_match.group(2):
                            test_name = summary_match.group(2)
                        if summary_match.group(3):
                            exception_type = summary_match.group(3)
                    else:
                        # FAILED something.py:12: Error
                        match = re.search(r"([^\s:]+\.py):(\d+):\s*([A-Za-z_][A-Za-z0-9_]*)?", parts)
                        if match:
                            file_path = match.group(1)
                            line_number = int(match.group(2))
                            if match.group(3):
                                exception_type = match.group(3)

                    failures.append(
                        FailureSignal(
                            test_name=test_name,
                            file_path=file_path,
                            line_number=line_number,
                            exception_type=exception_type,
                            stacktrace=line.strip(),
                            raw_output=raw_output,
                        )
                    )

        # If completely unrecognized but there are "E   Exception" or "File ..." lines, add a single fallback
        if not failures:
            e_match = re.search(r"^E\s+([A-Za-z_][A-Za-z0-9_]*)(?::|\s)", raw_output, re.MULTILINE)
            f_match = re.search(r'File "([^"]+)", line (\d+)', raw_output)
            if e_match or f_match:
                failures.append(
                    FailureSignal(
                        test_name="unknown",
                        file_path=f_match.group(1) if f_match else None,
                        line_number=int(f_match.group(2)) if f_match else None,
                        exception_type=e_match.group(1) if e_match else None,
                        stacktrace=raw_output.strip(),
                        raw_output=raw_output,
                    )
                )
        
        return failures
