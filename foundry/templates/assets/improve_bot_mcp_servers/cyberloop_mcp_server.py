#!/usr/bin/env python3
"""Cyberloop MCP tools exposed to the ImproveBot Claude agent."""

from __future__ import annotations

from mcp.server.fastmcp import FastMCP

mcp = FastMCP("cyberloop")


@mcp.tool()
def request_eval() -> str:
    """Request evaluation of the candidate bot saved at /output/bot/bot.py."""
    return "Evaluation requested."


if __name__ == "__main__":
    mcp.run()
