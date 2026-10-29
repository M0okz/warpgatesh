const announcement = document.querySelector('[role="status"]');
document.querySelectorAll("[data-copy]").forEach((button) => {
  button.addEventListener("click", async () => {
    const command = document
      .getElementById(button.dataset.copy)
      .textContent.trim();
    try {
      await navigator.clipboard.writeText(command);
      announcement.textContent = "Command copied to clipboard.";
      button.textContent = "Copied";
      setTimeout(() => {
        button.textContent = "Copy";
      }, 2000);
    } catch {
      announcement.textContent =
        "Unable to copy. Select the command and copy it manually.";
    }
  });
});
const menu = document.querySelector("[data-menu]");
const navigation = document.querySelector("#navigation");
function closeMenu() {
  menu.setAttribute("aria-expanded", "false");
  navigation.classList.remove("is-open");
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
