escape = |value| {
	bytes = Str.to_utf8(value)
	needs_escape = bytes.fold(
		Bool.False,
		|found, byte|
			if found {
				Bool.True
			} else {
				match byte {
					38 => Bool.True
					60 => Bool.True
					62 => Bool.True
					34 => Bool.True
					39 => Bool.True
					_ => Bool.False
				}
			},
	)
	if !needs_escape {
		value
	} else {
		escaped_bytes = bytes.fold(
			List.with_capacity(bytes.len() * 2),
			|out, byte|
				match byte {
					38 => out.append(38).append(97).append(109).append(112).append(59)
					60 => out.append(38).append(108).append(116).append(59)
					62 => out.append(38).append(103).append(116).append(59)
					34 => out.append(38).append(113).append(117).append(111).append(116).append(59)
					39 => out.append(38).append(35).append(51).append(57).append(59)
					_ => out.append(byte)
				},
		)
		match Str.from_utf8(escaped_bytes) {
			Ok(str) => str
			Err(_) => ""
		}
	}
}

join = |parts| Str.join_with(parts, "")

Html := [].{
	attribute = |name, value| " ${name}=\"${escape(value)}\""

	boolean_attribute = |name, enabled|
		if enabled {
			" ${name}"
		} else {
			""
		}

	dangerously_include_unescaped_html = |html| html

	element = |name, attrs, children|
		"<${name}${join(attrs)}>${join(children)}</${name}>"

	empty = ""

	fragment = |nodes| join(nodes)

	render = |html| html

	render_document = |html| "<!DOCTYPE html>\n${html}"

	render_fragment = |html| html

	render_without_doc_type = |html| html

	text = |value| escape(value)

	void_element = |name, attrs| "<${name}${join(attrs)} />"
}

## Clean text is returned unchanged (no-escape fast path).
expect {
	original = "Hello Roc"
	escape(original) == original
}

## Text and attributes escape `&`, `<`, `>`, `"`, and `'`.
expect {
	Html.text("&<>\"'") == "&amp;&lt;&gt;&quot;&#39;"
		and Html.attribute("title", "&<>\"'") == " title=\"&amp;&lt;&gt;&quot;&#39;\""
}

## CR and LF in attributes stay raw (not Node `&#13;` / `&#10;`).
expect {
	Html.attribute("title", "a\rb\nc") == " title=\"a\rb\nc\""
}

## False boolean_attribute omits the attribute.
expect {
	Html.boolean_attribute("disabled", Bool.False) == ""
		and Html.boolean_attribute("disabled", Bool.True) == " disabled"
}
