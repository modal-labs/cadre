import os
import traceback
from typing import Any

import httpx


class CadreException(Exception):
    pass


class Client:
    """Asynchronous client for Cadre.

    Read and write configuration templates from Cadre.

    Parameters
    ----------
    origin: str
        URL for Cadre service.
    timeout: float, default 0.3
        Request timeout.
    """
    def __init__(self, origin:str, timeout:float = 0.3):
        self.origin = origin
        self.timeout = timeout

    def _build_uri(self, path:str) -> str:
        return os.path.join(self.origin, path)

    async def _send(self, req: httpx.Request, parse_json:bool = True) -> httpx.AsyncClient:
        async with httpx.AsyncClient(base_url=self.origin, timeout=self.timeout) as client:
            try:
                res = await client.send(req)
            except httpx.TimeoutException:
                traceback.print_exc()
                raise CadreException("Cadre did respond within timeout deadline.")

            if res.status_code != 200:
                raise CadreException(f"Cadre responded with bad status code: {res.status_code}")

            if parse_json:
                return res.json()
            return res.text

    async def _get(self, uri:str) -> dict[str, Any]:        
        request = httpx.Request("GET", self._build_uri(uri))
        return await self._send(request)
    
    async def ping(self) -> str:
        """Verifies that Cadre is running by querying its healthcheck endpoint."""
        request = httpx.Request("GET", self._build_uri("ping"))
        return await self._send(request, parse_json=False)

    async def get_template(self, env:str) -> dict[str, Any]:
        return await self._get(f"t/{env}")
    
    async def load_config(self, env:str) -> dict[str, Any]:
        return await self._get(f"c/{env}")
    
    async def list_configs(self) -> dict[str, Any]:
        return await self._get(f"c")
    
    async def write_template(self, env:str, json:dict[str, Any]) -> dict[str, Any]:
        request = httpx.Request("PUT", self._build_uri(f"t/{env}"), json=json)
        return await self._send(request, parse_json=False)
