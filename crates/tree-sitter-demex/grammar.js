/**
 * @file Demex grammar for tree-sitter
 * @author Matteo Lutz <info@matteolutz.de>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "demex",

  extras: ($) => [
    /\s/, // whitespace
  ],

  conflicts: ($) => [
    // Conflict when parsing fixture_selector vs object (both can start with integer)
    [$.atomic_fixture_selector],
    // Conflict in create function - preset can optionally have as_clause
    [$.create_function],
    // Conflict in record function
    [$.record_function],
  ],

  rules: {
    // ==================== TOP-LEVEL RULES ====================
    command: ($) => choice($.function_call, $.object),

    // ==================== LEXICAL TOKENS ====================
    integer: ($) => /\d+/,
    float: ($) => /\d+\.\d+/,
    string: ($) => /"[^"]*"/,

    // ==================== KEYWORD ALIASES ====================
    group: ($) => choice("group", "g"),
    sequence: ($) => choice("sequence", "seq"),
    executor: ($) => choice("executor", "exec"),
    grandmaster: ($) => choice("grandmaster", "gm"),
    speedmaster: ($) => choice("speedmaster", "sm"),
    record_keyword: ($) => choice("record", "rec"),
    rename_keyword: ($) => choice("rename", "ren"),
    delete_keyword: ($) => choice("delete", "del"),
    highlight_keyword: ($) => choice("highlight", "hl"),
    unhighlight_keyword: ($) => choice("unhighlight", "uhl"),

    // ==================== FUNCTION (ACTION) DEFINITIONS ====================
    function_call: ($) =>
      choice(
        // fixture_selector , set_function
        seq($.fixture_selector, $.set_function),
        // discrete_set_function (attribute value on current selection)
        $.discrete_set_function,
        // "set" , set_property_function
        seq("set", $.set_property_function),
        // "home" , [ homeable_object ]
        seq("home", optional($.homeable_object)),
        // "record" , record_function
        seq($.record_keyword, $.record_function),
        // "create" , create_function
        seq("create", $.create_function),
        // "rename" , rename_function
        seq($.rename_keyword, $.rename_function),
        // "update" , update_function
        seq("update", $.update_function),
        // "delete" , delete_function
        seq($.delete_keyword, $.delete_function),
        // "move" , move_function
        seq("move", $.move_function),
        // "highlight" , [ fixture_selector ]
        seq($.highlight_keyword, optional($.fixture_selector)),
        // "unhighlight"
        $.unhighlight_keyword,
        // "clear"
        "clear",
        // "save"
        "save",
        // "lock"
        "lock",
        // "config" , config_type
        seq("config", $.config_type),
        // "test" , string
        seq("test", $.string),
        // "assign" , assign_function
        seq("assign", $.assign_function),
        // "unassign" , unassign_function
        seq("unassign", $.unassign_function),
        // "recall" , recall_function
        seq("recall", $.recall_function),
        // Easter egg commands
        "nuzul",
        "sueud",
        "grandetc",
      ),

    // ==================== SET FUNCTIONS ====================
    // Set function applied to a fixture selector
    set_function: ($) =>
      choice(
        $.specific_preset_or_range,
        seq($.attribute, $.channel_value),
      ),

    // Set function applied to current selection (no selector prefix)
    discrete_set_function: ($) => seq($.attribute, $.channel_value),

    // Set object property
    set_property_function: ($) => seq($.object, $.string, $.string),

    // ==================== RECORD FUNCTION ====================
    as_clause: ($) => seq("as", $.string),

    record_function: ($) =>
      choice(
        // "preset" , preset_id , "for" , fixture_selector , [ as_clause ] , [ "next" ]
        seq(
          "preset",
          $.preset_id,
          "for",
          $.fixture_selector,
          optional($.as_clause),
          optional("next"),
        ),
        // ( "group" | "g" ) , integer_or_next , "for" , fixture_selector , [ as_clause ]
        seq(
          $.group,
          $.integer_or_next,
          "for",
          $.fixture_selector,
          optional($.as_clause),
        ),
        // ( "sequence" | "seq" ) , integer , "cue" , cue_idx_or_next , "for" , fixture_selector , [ with_channel_type_selector ]
        seq(
          $.sequence,
          $.integer,
          "cue",
          $.cue_idx_or_next,
          "for",
          $.fixture_selector,
          optional($.with_channel_type_selector),
        ),
        // ( "executor" | "exec" ) , integer , [ cue_clause ] , "for" , fixture_selector , [ with_channel_type_selector ] , [ as_clause ]
        seq(
          $.executor,
          $.integer,
          optional($.cue_clause),
          "for",
          $.fixture_selector,
          optional($.with_channel_type_selector),
          optional($.as_clause),
        ),
      ),

    cue_clause: ($) =>
      choice(
        seq("cue", $.cue_idx_or_next),
        "next", // Legacy syntax for "cue next"
      ),

    with_channel_type_selector: ($) => seq("with", $.channel_type_selector),

    channel_type_selector: ($) =>
      choice(
        "all",
        "active",
        $.feature_type_list,
      ),

    // ==================== CREATE FUNCTION ====================
    create_function: ($) =>
      choice(
        // ( "sequence" | "seq" ) , integer_or_next , [ as_clause ]
        seq($.sequence, $.integer_or_next, optional($.as_clause)),
        // ( "executor" | "exec" ) , integer_or_next , "for" , ( "sequence" | "seq" ) , integer
        seq($.executor, $.integer_or_next, "for", $.sequence, $.integer),
        // "macro" , integer_or_next , "with" , command , [ as_clause ]
        seq("macro", $.integer_or_next, "with", $.command, optional($.as_clause)),
        // "preset" , float , [ as_clause ] (Create effect preset)
        seq("preset", $.float, optional($.as_clause)),
      ),

    // ==================== RENAME FUNCTION ====================
    rename_function: ($) => seq($.object, "to", $.string),

    // ==================== UPDATE FUNCTION ====================
    update_function: ($) =>
      choice(
        // "preset" , preset_id , "for" , fixture_selector , [ update_mode ]
        seq("preset", $.preset_id, "for", $.fixture_selector, optional($.update_mode)),
        // ( ( "sequence" | "seq" ) | ( "executor" | "exec" ) ) , integer , "cue" , discrete_cue_idx , "for" , fixture_selector , [ with_channel_type_selector ] , [ update_mode ]
        seq(
          choice($.sequence, $.executor),
          $.integer,
          "cue",
          $.discrete_cue_idx,
          "for",
          $.fixture_selector,
          optional($.with_channel_type_selector),
          optional($.update_mode),
        ),
      ),

    update_mode: ($) => choice("merge", "override"),

    // ==================== DELETE FUNCTION ====================
    delete_function: ($) => seq($.object_or_range, "really"),

    object_or_range: ($) => seq($.object, optional(seq("thru", $.object))),

    // ==================== MOVE FUNCTION ====================
    move_function: ($) => seq($.specific_preset, "to", $.specific_preset),

    // ==================== CONFIG FUNCTION ====================
    config_type: ($) => choice("output", "patch"),

    // ==================== ASSIGN FUNCTION ====================
    assign_function: ($) =>
      choice(
        // fixture_selector , "to" , button_id
        seq($.fixture_selector, "to", $.button_id),
        // ( "executor" | "exec" ) , integer , executor_assign_mode
        seq($.executor, $.integer, $.executor_assign_mode),
        // specific_preset_or_range , [ "with" , fixture_selector ] , "to" , button_id
        seq(
          $.specific_preset_or_range,
          optional(seq("with", $.fixture_selector)),
          "to",
          $.button_id,
        ),
        // "macro" , command , "to" , button_id
        seq("macro", $.command, "to", $.button_id),
        // ( "grandmaster" | "gm" ) , "to" , fader_id
        seq($.grandmaster, "to", $.fader_id),
        // ( "speedmaster" | "sm" ) , integer , speedmaster_assign_mode
        seq($.speedmaster, $.integer, $.speedmaster_assign_mode),
      ),

    executor_assign_mode: ($) =>
      choice(
        // "fader" , "to" , fader_id
        seq("fader", "to", $.fader_id),
        // "go" , "to" , button_id
        seq("go", "to", $.button_id),
        // "stop" , "to" , button_id
        seq("stop", "to", $.button_id),
        // "flash" , [ "stomp" ] , "to" , button_id
        seq("flash", optional("stomp"), "to", $.button_id),
      ),

    speedmaster_assign_mode: ($) =>
      choice(
        // "fader" , "to" , fader_id
        seq("fader", "to", $.fader_id),
        // "tap" , "to" , button_id
        seq("tap", "to", $.button_id),
      ),

    button_id: ($) => $.float, // device_idx.button_id
    fader_id: ($) => $.float, // device_idx.fader_id

    // ==================== UNASSIGN FUNCTION ====================
    unassign_function: ($) =>
      choice(
        seq("button", $.button_id),
        seq("fader", $.fader_id),
      ),

    // ==================== RECALL FUNCTION ====================
    recall_function: ($) =>
      seq($.sequence, $.integer, "cue", $.discrete_cue_idx),

    // ==================== OBJECT DEFINITIONS ====================
    object: ($) =>
      choice(
        $.homeable_object,
        $.specific_preset,
        seq($.sequence, $.integer, optional(seq("cue", $.discrete_cue_idx))),
        $.specific_macro,
      ),

    homeable_object: ($) =>
      choice(
        $.fixture_selector,
        $.specific_executor,
        "programmer",
      ),

    specific_preset: ($) => seq("preset", $.preset_id),
    specific_preset_or_range: ($) =>
      seq($.specific_preset, optional(seq("thru", $.specific_preset))),

    // ==================== PRESET IDENTIFIERS ====================
    // Preset ID can be specified as:
    // - float literal (e.g., "1.5" = feature_group 1, preset 5)
    // - string feature group name followed by integer (e.g., "color" 5)
    preset_id: ($) =>
      choice(
        $.float,
        seq($.string, $.integer),
      ),

    // ==================== FIXTURE SELECTOR ====================
    fixture_selector: ($) =>
      seq($.atomic_fixture_selector, optional($.fixture_selector_operation)),

    fixture_selector_operation: ($) =>
      choice(
        // "+" , fixture_selector (Union: add fixtures)
        seq("+", $.fixture_selector),
        // "-" , fixture_selector (Difference: subtract fixtures)
        seq("-", $.fixture_selector),
        // "%" , [ "!" ] , integer (Modulus)
        seq("%", optional("!"), $.integer),
      ),

    atomic_fixture_selector: ($) =>
      choice(
        // "~" (Current selection)
        "~",
        // integer , [ "thru" , integer ] (Single fixture or range)
        seq($.integer, optional(seq("thru", $.integer))),
        // float , [ "thru" , integer ] (Fixture path or path range)
        seq($.float, optional(seq("thru", $.integer))),
        // ( "group" | "g" ) , integer (Fixture group)
        $.specific_group,
        // "(" , fixture_selector , ")" (Grouping)
        seq("(", $.fixture_selector, ")"),
      ),

    // ==================== ATTRIBUTE AND VALUE DEFINITIONS ====================
    attribute: ($) =>
      choice(
        "intens",
        "@",
        $.string, // Named attribute
      ),

    channel_value: ($) =>
      choice(
        seq($.discrete_channel_value, optional(seq("thru", $.discrete_channel_value))),
        "home",
      ),

    discrete_channel_value: ($) =>
      choice(
        $.float,
        "full",
        "half",
        "out",
      ),

    // ==================== FEATURE TYPES ====================
    feature_type_list: ($) =>
      seq("(", optional(seq($.feature_type, repeat(seq(",", $.feature_type)))), ")"),

    feature_type: ($) => "intens", // Currently only intens/dimmer supported

    // ==================== CUE INDEX ====================
    cue_idx_or_next: ($) =>
      choice(
        $.discrete_cue_idx,
        "next",
      ),

    discrete_cue_idx: ($) =>
      choice(
        $.integer,
        $.float,
      ),

    // ==================== HELPER RULES ====================
    integer_or_next: ($) =>
      choice(
        $.integer,
        "next",
      ),

    // ==================== SPECIFIC OBJECTS ====================
    specific_sequence: ($) => seq($.sequence, $.integer),
    specific_sequence_cue: ($) => seq($.specific_sequence, "cue", $.discrete_cue_idx),
    specific_macro: ($) => seq("macro", $.integer),
    specific_executor: ($) => seq($.executor, $.integer),
    specific_group: ($) => seq($.group, $.integer),

    // Backward compatibility aliases
    channel_attribute: ($) => $.attribute,
  },
});
