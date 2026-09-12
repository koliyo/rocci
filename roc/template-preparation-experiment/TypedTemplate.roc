import Template
import Value

## Experimental adapter over the pinned Templegen engine, not a product API.
## One type variable connects the sample and the returned renderer's input.
TypedTemplate :: [].{
	prepare : Str, Str, a -> (a -> Str) where [a.encoder_for : Value.Encoding -> (a, Value.State -> Try(Value.State, []))]
	prepare = |source_name, source, sample| {
		parsed = Template.bag([(source_name, source)]).template(source_name)
		checked = match Template.check_value(parsed, Value.from(sample)) {
			Ok(template) => template
			Err(message) => crash "${source_name}: ${message}"
		}
		|context| checked.render(context)
	}
}
