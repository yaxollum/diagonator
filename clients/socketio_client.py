#!/usr/bin/env python3

import asyncio

import socketio


async def main():
    async with socketio.AsyncSimpleClient() as sio:
        await sio.connect("http://localhost:3000")
        while True:
            event = await sio.receive()
            if event[0] == "info_update":
                print(event[1])


asyncio.run(main())
