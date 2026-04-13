#!/usr/bin/env python3
import argparse
import json
import sys
import urllib.request


def parse_args():
    parser = argparse.ArgumentParser(description="Submit a liquidation proof to /open_zyga_position")
    parser.add_argument("--file", default="circuits/liquidation_proof.json")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=3000)
    parser.add_argument("--path", default="/open_zyga_position")
    parser.add_argument("--user-id", type=int, default=1)
    parser.add_argument("--direction", choices=["long", "short"], default="long")
    parser.add_argument("--entry-price", type=int, default=1000000)
    parser.add_argument("--notional", type=int, default=1000000)
    parser.add_argument("--leverage", type=int, default=1)
    parser.add_argument("--initial-margin", type=int, default=100000)
    return parser.parse_args()


def main():
    args = parse_args()

    with open(args.file) as f:
        proof = json.load(f)

    public_inputs = proof.get("public_inputs", {})
    mark_price = public_inputs.get("mark_price")
    if mark_price is None:
        print("proof file is missing public_inputs.mark_price", file=sys.stderr)
        return 2

    body = {
        "user_id": args.user_id,
        "direction": args.direction == "long",
        "entry_price": args.entry_price,
        "notional": args.notional,
        "leverage": args.leverage,
        "initial_margin": args.initial_margin,
        "proof": {
            "mark_price": int(mark_price),
            "public_inputs": public_inputs,
            "pairing_proof": proof.get("pairing_proof"),
        },
    }

    req = urllib.request.Request(
        f"http://{args.host}:{args.port}{args.path}",
        data=json.dumps(body).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    with urllib.request.urlopen(req) as resp:
        payload = resp.read().decode("utf-8")
        print(resp.status)
        print(payload)


if __name__ == "__main__":
    raise SystemExit(main())
