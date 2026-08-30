import os
from dataclasses import dataclass


ORIGIN_ROOT_DEFAULT = "/srv/rocci/prod"
HTTP_PORT_DEFAULT = "8080"
COMPOSE_PROJECT_DEFAULT = "rocci-prod"
IMAGE_TAG_DEFAULT = "local"


@dataclass(frozen=True)
class LanePreset:
    origin_root: str
    http_port: str
    compose_project: str
    publish_live: bool
    image_tag: str


LANES: dict[str, LanePreset] = {
    "production": LanePreset(
        origin_root="/srv/rocci/prod",
        http_port="8080",
        compose_project="rocci-prod",
        publish_live=False,
        image_tag="prod",
    ),
    "staging": LanePreset(
        origin_root="/srv/rocci/staging",
        http_port="8081",
        compose_project="rocci-staging",
        publish_live=True,
        image_tag="staging",
    ),
}


@dataclass(frozen=True)
class LaneConfig:
    name: str | None
    origin_root: str
    http_port: str
    compose_project: str
    publish_live: bool | None
    image_tag: str
    bootstrap_dest: str


def _pick(key: str, default: str) -> str:
    value = os.environ.get(key)
    if value is None or value == "":
        return default
    return value


def parse_publish_live(raw: str) -> bool:
    return raw.strip().lower() not in {"0", "false", "no"}


def resolved_lane() -> LaneConfig:
    name = os.environ.get("ROCCI_LANE", "").strip() or None
    if name is not None and name not in LANES:
        raise SystemExit(f"error: unknown ROCCI_LANE={name!r}")
    preset = LANES[name] if name is not None else None
    if preset is not None:
        origin = _pick("ROCCI_ORIGIN_ROOT", preset.origin_root)
        publish: bool | None = preset.publish_live
        if "ROCCI_PUBLISH_LIVE" in os.environ:
            publish = parse_publish_live(os.environ["ROCCI_PUBLISH_LIVE"])
        return LaneConfig(
            name=name,
            origin_root=origin,
            http_port=_pick("ROCCI_HTTP_PORT", preset.http_port),
            compose_project=_pick("COMPOSE_PROJECT_NAME", preset.compose_project),
            publish_live=publish,
            image_tag=_pick("ROCCI_IMAGE_TAG", preset.image_tag),
            bootstrap_dest=_pick("ROCCI_BOOTSTRAP_DEST", f"{origin}/docker"),
        )
    origin = _pick("ROCCI_ORIGIN_ROOT", ORIGIN_ROOT_DEFAULT)
    publish = None
    if "ROCCI_PUBLISH_LIVE" in os.environ:
        publish = parse_publish_live(os.environ["ROCCI_PUBLISH_LIVE"])
    return LaneConfig(
        name=None,
        origin_root=origin,
        http_port=_pick("ROCCI_HTTP_PORT", HTTP_PORT_DEFAULT),
        compose_project=_pick("COMPOSE_PROJECT_NAME", COMPOSE_PROJECT_DEFAULT),
        publish_live=publish,
        image_tag=_pick("ROCCI_IMAGE_TAG", IMAGE_TAG_DEFAULT),
        bootstrap_dest=_pick("ROCCI_BOOTSTRAP_DEST", f"{origin}/docker"),
    )


def should_publish_live(live_ids: list[str], cfg: LaneConfig | None = None) -> bool:
    if not live_ids:
        return False
    config = cfg if cfg is not None else resolved_lane()
    if config.publish_live is None:
        return True
    return config.publish_live
