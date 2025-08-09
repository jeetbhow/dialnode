type Project = {
  dir: string;
};

const project = $state({
  dir: ""
});

export function setProjectDirectory(dir: string) {
  project.dir = dir;
}

export function useProject(): Project {
  return project;
}
