import asyncio
import json
import os
import subprocess
from contextlib import asynccontextmanager
from random import randint
from tempfile import TemporaryDirectory
from typing import Union

import pytest

import cadre

TEST_ENVIRONMENT: str = "test"
CONFIG_TEMPLATE:dict[str, Union[str, dict[str]]] = {
        "a": "a",
        "b": "b",
        "c": {
            "a": "a",
            "b": "b"
        },
    }


@asynccontextmanager
async def start_cadre():
    
    # Builds cadre binary.
    subprocess.run("cargo build --release --manifest-path ../Cargo.toml".split())

    with TemporaryDirectory() as tmpdir:

        # Write test config template.
        filename = os.path.join(tmpdir, f"{TEST_ENVIRONMENT}.json")
        with open(filename, "w") as f:
            json.dump(CONFIG_TEMPLATE, f)

        # Start Cadre server.
        port = 7000 + randint(0, 100)
        cmd = [
            "../target/release/cadre",
            "--port",
            str(port),
            "--local-dir",
            str(tmpdir)
        ]

        print(f"running cadre => {' '.join(cmd)}")
        proc = await asyncio.create_subprocess_exec(*cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
        yield f"http://localhost:{port}"

    # Terminate process and log any potential error.
    proc.kill()
    stdout, stderr = await proc.communicate()
    print(f'[{cmd!r} exited with {proc.returncode}]')
    if stdout:
        print(f'[stdout]\n{stdout.decode()}')
    if stderr:
        print(f'[stderr]\n{stderr.decode()}')


@pytest.mark.asyncio
async def test_read():
    async with start_cadre() as cadre_url:
        client = cadre.Client(origin=cadre_url)

        assert "ok" in await client.ping()
        assert "a" == (await client.get_template(TEST_ENVIRONMENT)).get("a")
        assert "a" == (await client.load_config(TEST_ENVIRONMENT)).get("a")
        assert "test" in await client.list_configs()


@pytest.mark.asyncio
async def test_write():
    async with start_cadre() as cadre_url:
        client = cadre.Client(origin=cadre_url)
    
        environment = "write_test"
        await client.write_template(env=environment, json={"test": "foo"})
        assert "foo" == (await client.get_template(environment)).get("test")
        assert "foo" == (await client.load_config(environment)).get("test")
        assert environment in await client.list_configs()
