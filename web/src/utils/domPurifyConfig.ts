import DOMPurify from "dompurify";

// Configure DOMPurify hook globally to prevent reverse tabnabbing
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (
    node instanceof Element &&
    node.tagName === "A" &&
    node.getAttribute("target") === "_blank"
  ) {
    const currentRel = node.getAttribute("rel") || "";
    let newRel = currentRel;
    if (!newRel.includes("noopener")) newRel += " noopener";
    if (!newRel.includes("noreferrer")) newRel += " noreferrer";
    node.setAttribute("rel", newRel.trim());
  }
});

export default DOMPurify;
