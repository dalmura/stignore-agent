import json

import pytest


def test_content_types(agent):
    response = agent.client.get("/api/v1/discover")
    assert response.status == "200 OK"

    recieved = json.loads(response.data)

    expected = {
        "ok": True,
        "content_types": [
            {"name": "share-1"},
            {"name": "share-2"},
        ],
    }

    assert recieved == expected


def test_content_type_listing(agent):
    response = agent.client.get("/api/v1/share-1/listing")
    assert response.status == "200 OK"

    recieved = json.loads(response.data)

    expected = {
        "ok": True,
        "folders": [
            {
                "name": "Object 1",
                "size_megabytes": 25,
            },
            {
                "name": "Object 2",
                "size_megabytes": 12,
            },
            {
                "name": "Object 3",
                "size_megabytes": 5,
            },
        ],
    }

    assert recieved == expected


def test_stignore_listing(agent):
    stignore_path = agent.config["base_folder"] / "share-1" / ".stignore"
    stignore_path.write_text("Object 1/\nObject 2/\n!Object 3/\n")

    response = agent.client.get("/api/v1/share-1/stignore")
    assert response.status == "200 OK"

    recieved = json.loads(response.data)

    expected = {
        "ok": True,
        "entries": [
            {
                "raw": "Object 1/",
                "name": "Object 1",
                "ignore_type": "ignore",
            },
            {
                "raw": "Object 2/",
                "name": "Object 2",
                "ignore_type": "ignore",
            },
            {
                "raw": "!Object 3/",
                "name": "Object 3",
                "ignore_type": "keep",
            },
        ],
    }

    expected["entries"].sort(key=lambda x: x["raw"])

    assert recieved == expected
