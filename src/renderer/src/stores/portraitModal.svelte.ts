import type { Portrait, DbRequestType, Speaker } from '../types';

type PortraitModalStore = {
  open: boolean;
  requestType?: DbRequestType;
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

let _resolve: (result: PortraitResult | SpeakerResult) => void | null = null;

export const modal: PortraitModalStore = $state({
  open: false,
  nodeId: null
});

export function requestModal(
  forNodeId: string,
  type: DbRequestType
): Promise<PortraitResult | SpeakerResult> {
  modal.open = true;
  modal.nodeId = forNodeId;
  modal.requestType = type;

  return new Promise((resolve) => {
    _resolve = resolve;
  });
}

export function fufillModal(value: Portrait | Speaker) {
  if (_resolve) {
    _resolve({ nodeId: modal.nodeId, value });
    _resolve = null;
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
