const announcement = document.querySelector('[role="status"]');
const copyTimers = new WeakMap();
const pendingCopies = new WeakSet();
document.querySelectorAll("[data-copy]").forEach((button) => {
  button.addEventListener("click", async () => {
    if (pendingCopies.has(button)) return;
    pendingCopies.add(button);
    clearTimeout(copyTimers.get(button));
    announcement.textContent = "";
    const line = button.closest(".copy-line");
    line.querySelector(".copy-error")?.remove();
    button.removeAttribute("aria-describedby");
    const command = document
      .getElementById(button.dataset.copy)
      .textContent.trim();
    try {
      await navigator.clipboard.writeText(command);
      announcement.textContent = "Command copied to clipboard.";
      button.textContent = "Copied";
      button.dataset.copyState = "copied";
    } catch {
      button.textContent = "Retry";
      button.dataset.copyState = "error";
      const guidance = document.createElement("p");
      guidance.className = "copy-error";
      guidance.id = `${button.dataset.copy}-error`;
      guidance.textContent =
        "Select the command and copy it manually, or try again.";
      line.append(guidance);
      button.setAttribute("aria-describedby", guidance.id);
      announcement.textContent =
        "Unable to copy. Select the command and copy it manually.";
    } finally {
      pendingCopies.delete(button);
      if (button.dataset.copyState === "copied") {
        copyTimers.set(
          button,
          setTimeout(() => {
            button.textContent = "Copy";
            delete button.dataset.copyState;
          }, 2000),
        );
      }
    }
  });
});
const menu = document.querySelector("[data-menu]");
const navigation = document.querySelector("#navigation");
const mobileLayout = matchMedia("(max-width: 700px)");
document.documentElement.classList.add("js");
function closeMenu() {
  menu?.setAttribute("aria-expanded", "false");
  navigation?.classList.remove("is-open");
}
menu?.addEventListener("click", () => {
  const open = menu.getAttribute("aria-expanded") !== "true";
  menu.setAttribute("aria-expanded", String(open));
  navigation.classList.toggle("is-open", open);
});
navigation
  ?.querySelectorAll("a")
  .forEach((link) => link.addEventListener("click", closeMenu));
document.addEventListener("keydown", (event) => {
  if (
    event.key === "Escape" &&
    menu?.getAttribute("aria-expanded") === "true"
  ) {
    closeMenu();
    menu.focus();
  }
});
document.addEventListener("click", (event) => {
  if (
    menu?.getAttribute("aria-expanded") !== "true" ||
    menu.closest("header").contains(event.target)
  ) return;
  const restoreFocus = navigation.contains(document.activeElement);
  closeMenu();
  if (restoreFocus) menu.focus();
});
mobileLayout.addEventListener("change", () => {
  const active = document.activeElement;
  closeMenu();
  if (mobileLayout.matches && navigation?.contains(active)) menu.focus();
  else if (!mobileLayout.matches && active === menu) {
    navigation?.querySelector("a")?.focus();
  }
});
const overflowObserver = new ResizeObserver((entries) => {
  for (const { target: region } of entries) {
    if (region.scrollWidth > region.clientWidth) {
      region.tabIndex = 0;
      if (region.tagName === "PRE") region.setAttribute("role", "region");
      region.setAttribute("aria-label",
        region.tagName === "PRE" ? "Code example" : "Documentation table");
    } else {
      region.removeAttribute("tabindex");
      if (region.tagName === "PRE") region.removeAttribute("role");
      region.removeAttribute("aria-label");
    }
  }
});
document.querySelectorAll("pre, .docs-content table")
  .forEach((region) => overflowObserver.observe(region));
