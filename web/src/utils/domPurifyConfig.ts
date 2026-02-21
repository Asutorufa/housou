import DOMPurify from "dompurify";

// Configure DOMPurify hook globally to prevent reverse tabnabbing
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (
    node instanceof Element &&
    node.tagName === "A" &&
    node.getAttribute("target")?.toLowerCase() === "_blank"
  ) {
    const currentRel = node.getAttribute("rel") || "";
    const rels = new Set(currentRel.split(/\s+/).filter(Boolean));
    rels.add("noopener");
    rels.add("noreferrer");
    node.setAttribute("rel", Array.from(rels).join(" "));
  }
});

export default DOMPurify;
