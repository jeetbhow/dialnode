import type { DbEntity, DbEntityKind, Portrait, Skill, Speaker } from "../../../shared/types";

type PortraitModalStore = {
  open: boolean;
  requestType?: DbEntityKind;
  nodeId?: string;
};

type PortraitResult = {
  nodeId: string;
  value: Portrait;
};

type SpeakerResult = {
  nodeId: string;
  value: Speaker;
};

type SkillResult = {
  nodeId: string;
  value: Skill;
};

let _resolve: ((result: PortraitResult | SpeakerResult | SkillResult) => void) | null = null;

export const modal: PortraitModalStore = $state({
  open: false,
  requestType: null,
  nodeId: null
});

export function requestModal(
  forNodeId: string,
  type: DbEntityKind
): Promise<PortraitResult | SpeakerResult | SkillResult> {
  modal.open = true;
  modal.nodeId = forNodeId;
  modal.requestType = type;

  return new Promise((resolve) => {
    _resolve = resolve;
  });
}

export function fulfillModal(entity: DbEntity) {
  if (!_resolve) {
    return;
  }

  switch (entity.kind) {
    case "portrait":
      _resolve({ nodeId: modal.nodeId, value: entity as Portrait });
      break;
    case "speaker":
      _resolve({ nodeId: modal.nodeId, value: entity as Speaker });
      break;
    case "skill":
      _resolve({ nodeId: modal.nodeId, value: entity as Skill });
      break;
  }

  modal.open = false;
  modal.nodeId = null;
  modal.requestType = null;
}

export function cancelModal() {
  if (_resolve) {
    _resolve(null);
    _resolve = null;
  }
  modal.open = false;
  modal.nodeId = null;
  modal.requestType = null;
}
