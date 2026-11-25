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

  rules: {
    command: ($) => choice($.function_call, $.object),

    integer: ($) => /\d+/,
    float: ($) => /([0-9]*[.])?[0-9]+/,
    string: ($) => /"(.*)"/,

    group: ($) => choice("group", "g"),
    sequence: ($) => choice("sequence", "seq"),
    executor: ($) => choice("executor", "exec"),

    function_call: ($) =>
      choice(
        seq($.fixture_selector, $.set_function_call),
        seq("set", $.set_property_function),
        seq("home", optional($.homeable_object)),
        "clear",

        seq("record", $.record_function),

        "save",
        "lock",

        seq("test", $.string),

        "nuzul",
        "sueud",
        "grandetc",
      ),

    set_function_call: ($) =>
      choice(
        seq($.channel_attribute, $.channel_value),
        $.specific_preset_or_range,
      ),

    set_property_function: ($) => seq($.object, $.string, $.string),

    as: ($) => seq("as", $.string),
    record_function: ($) =>
      choice(
        seq(
          alias(seq("preset", choice($.preset_id, "next")), $.object),
          "for",
          $.fixture_selector,
          optional($.as),
          optional("next"),
        ),

        seq(
          alias(seq($.group, choice($.integer, "next")), $.object),
          "for",
          $.fixture_selector,
          optional($.as),
        ),

        seq(
          alias(
            seq($.sequence, $.integer, "cue", choice($.float, "next")),
            $.object,
          ),
          "for",
          $.fixture_selector,
        ),

        seq(
          alias(seq($.executor, $.integer), $.object),
          optional(seq("cue", choice($.float, "next"))),
          "for",
          $.fixture_selector,
          optional($.as),
        ),
      ),

    object: ($) =>
      choice(
        $.homeable_object,
        $.specific_preset,
        $.specific_sequence,
        $.specific_sequence_cue,
      ),

    homeable_object: ($) => choice($.fixture_selector, $.specific_executor),

    fixture_selector: ($) =>
      choice(
        $.atomic_fixture_selector,
        seq(
          $.atomic_fixture_selector,
          repeat(seq("+", $.atomic_fixture_selector)),
        ),
        seq(
          $.atomic_fixture_selector,
          repeat(seq("-", $.atomic_fixture_selector)),
        ),
        seq(
          $.atomic_fixture_selector,
          repeat(seq("%", $.atomic_fixture_selector)),
        ),
      ),

    atomic_fixture_selector: ($) =>
      choice(
        $.integer,
        seq($.integer, "thru", $.integer),
        $.specific_group,
        seq("(", $.fixture_selector, ")"),
        "~",
      ),

    preset_id: ($) => seq($.integer, ".", $.integer),
    specific_preset: ($) => seq("preset", $.preset_id),
    specific_preset_or_range: ($) =>
      choice(
        $.specific_preset,
        seq($.specific_preset, "thru", $.specific_preset),
      ),

    specific_sequence: ($) => seq($.sequence, $.integer),
    specific_sequence_cue: ($) => seq($.specific_sequence, "cue", $.integer),
    specific_macro: ($) => seq("macro", $.integer),
    specific_executor: ($) => seq($.executor, $.integer),
    specific_group: ($) => seq($.group, $.integer),

    channel_attribute: ($) => choice("intens", "@"),
    channel_value: ($) =>
      seq(
        $.discrete_channel_value,
        optional(seq("thru", $.discrete_channel_value)),
      ),
    discrete_channel_value: ($) => choice($.float, "full", "half", "out"),
  },
});
