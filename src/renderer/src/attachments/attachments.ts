import { type Attachment } from "svelte/attachments";

export function clickOutside(callback: () => void): Attachment {
  return (elem: HTMLElement) => {
    function handleClick(event: MouseEvent): void {
      if (!elem.contains(event.target as HTMLElement)) {
        callback();
      }
    }

    document.addEventListener("click", handleClick);

    return () => document.removeEventListener("click", handleClick);
  };
}
