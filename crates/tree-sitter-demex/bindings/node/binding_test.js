const assert = require("node:assert");
const { test } = require("node:test");

test("can load grammar", () => {
  const lang = require(".");
  // Check that the language object is loaded correctly
  assert.ok(lang.language, "Language should be defined");
  assert.ok(lang.nodeTypeInfo, "Node type info should be defined");
  // Verify some expected node types are present
  const nodeTypeNames = lang.nodeTypeInfo.map(t => t.type);
  assert.ok(nodeTypeNames.includes("command"), "Should have 'command' node type");
  assert.ok(nodeTypeNames.includes("function_call"), "Should have 'function_call' node type");
  assert.ok(nodeTypeNames.includes("fixture_selector"), "Should have 'fixture_selector' node type");
});
