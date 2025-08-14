type Project = {
  dir: string;
};

const project = $state({
  dir: ""
});

export function setProjectDirectory(dir: string): void {
  project.dir = dir;
}

export function useProject(): Project {
  return project;
}
