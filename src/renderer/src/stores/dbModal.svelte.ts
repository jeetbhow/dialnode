import type { DbEntity, DbEntityKind, Portrait, Skill, Speaker } from "../../../shared/types";

type DbModalRequest = {
  type: DbEntityKind;
  nodeId: string;
};

type PortraitModalStore = {
  open: boolean;
  request: DbModalRequest | null;
};

type PortraitResult = {
  request: DbModalRequest;
  value: Portrait;
};

type SpeakerResult = {
  request: DbModalRequest;
  value: Speaker;
};

type SkillResult = {
  request: DbModalRequest;
  value: Skill;
};

type ModalRequestResult = PortraitResult | SpeakerResult | SkillResult | null;

let _resolve: ((result: ModalRequestResult) => void) | null = null;

export const modal: PortraitModalStore = $state({
  open: false,
  request: null
});

export function requestModal(nodeId: string, type: DbEntityKind): Promise<ModalRequestResult> {
  if (_resolve) {
    _resolve(null);
    _resolve = null;
  }

  modal.open = true;
  modal.request = { nodeId, type };

  return new Promise((resolve) => {
    _resolve = resolve;
  });
}

export function fulfillModal(entity: DbEntity): void {
  if (!_resolve || !modal.request) {
    return;
  }

  switch (entity.kind) {
    case "portrait":
      _resolve({ request: modal.request, value: entity });
      break;
    case "speaker":
      _resolve({ request: modal.request, value: entity });
      break;
    case "skill":
      _resolve({ request: modal.request, value: entity });
      break;
  }

  _resolve = null;
  modal.open = false;
  modal.request = null;
}

export function cancelModal(): void {
  if (_resolve) {
    _resolve(null);
    _resolve = null;
  }

  modal.open = false;
  modal.request = null;
}
