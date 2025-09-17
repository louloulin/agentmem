"""
AgentMem Python SDK

Official Python client library for AgentMem - Enterprise-grade memory management for AI agents.
"""

from .client import AgentMemClient
from .types import (
    Memory,
    MemoryType,
    SearchQuery,
    SearchResult,
    MemoryStats,
    AgentMemError,
    AuthenticationError,
    ValidationError,
    NetworkError,
)
from .config import Config

__version__ = "6.0.0"
__author__ = "AgentMem Team"
__email__ = "support@agentmem.dev"

__all__ = [
    "AgentMemClient",
    "Memory",
    "MemoryType", 
    "SearchQuery",
    "SearchResult",
    "MemoryStats",
    "AgentMemError",
    "AuthenticationError",
    "ValidationError",
    "NetworkError",
    "Config",
]
