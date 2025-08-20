import type { Repository } from "../../../shared/types";

const repository = $state<Repository>(null);

export function setRepository(repository: Repository) {
  repository = repository;
}

export function setGodotProjectLocation(location: string): void {
  repository.godotProjectLocation = location;
}

export function useRepository(): Repository {
  return repository;
}
