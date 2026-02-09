; Literals
(integer) @number
(float) @number
(string) @string

; Functions
(function_call) @function
(set_function) @function
(discrete_set_function) @function
(record_function) @function
(create_function) @function
(rename_function) @function
(update_function) @function
(delete_function) @function
(move_function) @function
(assign_function) @function
(unassign_function) @function
(recall_function) @function

; Objects
(object) @type
(homeable_object) @type
(specific_preset) @type
(specific_sequence) @type
(specific_sequence_cue) @type
(specific_macro) @type
(specific_executor) @type
(specific_group) @type

; Selectors
(fixture_selector) @type
(atomic_fixture_selector) @type
(fixture_selector_operation) @operator

; Attributes and values
(attribute) @attribute
(channel_attribute) @attribute
(channel_value) @number
(discrete_channel_value) @number

; Keywords (action)
(record_keyword) @keyword
(rename_keyword) @keyword
(delete_keyword) @keyword
(highlight_keyword) @keyword
(unhighlight_keyword) @keyword

; Keywords (objects)
(group) @keyword
(sequence) @keyword
(executor) @keyword
(grandmaster) @keyword
(speedmaster) @keyword

; Keyword literals
"home" @keyword
"set" @keyword
"create" @keyword
"update" @keyword
"move" @keyword
"clear" @keyword
"save" @keyword
"lock" @keyword
"config" @keyword
"test" @keyword
"assign" @keyword
"unassign" @keyword
"recall" @keyword
"nuzul" @keyword
"sueud" @keyword
"grandetc" @keyword
"preset" @keyword
"macro" @keyword
"cue" @keyword
"programmer" @keyword
"button" @keyword
"fader" @keyword

; Other keywords
"for" @keyword
"as" @keyword
"to" @keyword
"with" @keyword
"thru" @keyword
"all" @keyword
"active" @keyword
"merge" @keyword
"override" @keyword
"really" @keyword
"next" @keyword
"output" @keyword
"patch" @keyword
"go" @keyword
"stop" @keyword
"flash" @keyword
"stomp" @keyword
"tap" @keyword
"intens" @keyword

; Value keywords
"full" @constant
"half" @constant
"out" @constant

; Punctuation
"(" @punctuation.bracket
")" @punctuation.bracket
"," @punctuation.delimiter

; Operators
"+" @operator
"-" @operator
"%" @operator
"!" @operator
"@" @operator
"~" @operator
