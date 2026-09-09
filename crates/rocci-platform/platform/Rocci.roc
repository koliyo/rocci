import Host
import Path
import IOErr exposing [IOErr]

## Hosted parse/compile over crates/rocci-template. This is not `rocci run`
## and does not interpret interpolations.
Rocci := [].{

	Diagnostic : { code : Str, message : Str, start : U64, end : U64 }

	Source : { name : Str, source : Str }

	CompileResult : { roc : Str, diagnostics : List(Diagnostic) }

	ParseResult : { ast : Str, diagnostics : List(Diagnostic) }

	compile! : Source => CompileResult
	compile! = |input| Host.rocci_compile!(input)

	parse! : Source => ParseResult
	parse! = |input| Host.rocci_parse!(input)

	compile_file! : Path.Path => Try(CompileResult, [PathErr(IOErr), ..])
	compile_file! = |path| {
		source = Path.read_utf8!(path)?
		Ok(compile!({ name: Path.display(path), source: source }))
	}

	parse_file! : Path.Path => Try(ParseResult, [PathErr(IOErr), ..])
	parse_file! = |path| {
		source = Path.read_utf8!(path)?
		Ok(parse!({ name: Path.display(path), source: source }))
	}
}
