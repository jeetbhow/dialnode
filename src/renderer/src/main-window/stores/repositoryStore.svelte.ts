import type { Repository } from "../../../../shared/types";

let repository = $state<Repository>(null);

export function setGodotProjectLocation(location: string): void {
  repository.godotProjectLocation = location;
}

export function useRepository(): Repository {
  return repository;
}

export async function fetchRepository() {
  repository = await window.api.waitForRepo();
}