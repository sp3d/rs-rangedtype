#![crate_name = "rangedtype"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "WTFPL"]

#![feature(plugin_registrar, managed_boxes, macro_rules)]

extern crate syntax;
extern crate rustc;

use rustc::plugin::Registry;
use syntax::ast;
use syntax::ext::base;
use syntax::ext::base::{ExtCtxt, MacItem};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::codemap::{Span, mk_sp};
use std::gc::{Gc, GC};

#[macro_export]
macro_rules! ranged_type_impl_inner( ($ident: ident, $Which: ident, $which: ident, $CheckedWhich: ident, $checked_which: ident, $checked_which_internal: ident) => (
	impl $ident
	{
		fn $checked_which_internal (&self, y: &$ident) -> Option<$ident>
		{
			let sv = &(*self as int);
			let yv = &(*y as int);
			match sv. $checked_which (yv)
			{
				Some(n) => std::num::FromPrimitive::from_int(n),
				_ => None,
			}
		}
	}
	impl $Which<$ident,$ident> for $ident
	{
		fn $which(&self, y: &$ident) -> $ident
		{
			let min: $ident = std::num::Bounded::min_value();
			let max: $ident = std::num::Bounded::max_value();
			match self. $checked_which_internal (y)
			{
				Some(x) => x,
				None => fail!("result {} - {} lies out of range [{}, {}] for bounded type", *self, *y, min, max),
			}
		}
	}
	impl $CheckedWhich for $ident
	{
		fn $checked_which (&self, y: &$ident) -> Option<$ident>
		{
			self. $checked_which_internal (y)
		}
	}
))

#[macro_export]
macro_rules! ranged_type( ($ident: ident, $lower: expr, $upper: expr) => (
	ranged_type_enumdef!($ident, $lower, $upper)
	impl $ident
	{
		#[inline(always)]
		unsafe fn from_primitive_internal(x: i64) -> $ident
		{
			if cfg!(target_endian = "little")
			{
				std::mem::transmute_copy(&x)
			}
			else
			{
				//TODO: verify that this is correct on big-endian
				//out_width will always be >= in_width since we transmute from i64 to a smaller enum
				let out_width = std::mem::size_of::<$ident>();
				let in_width = std::mem::size_of::<i64>();
				let addr = &x as *const _ as uint + in_width - out_width;
				std::mem::transmute_copy(&*(addr as *const i64))
			}
		}
	}
	impl std::num::Bounded for $ident
	{
		fn min_value() -> $ident {unsafe {$ident::from_primitive_internal($lower)}}
		fn max_value() -> $ident {unsafe {$ident::from_primitive_internal($upper)}}
	}
	impl std::num::FromPrimitive for $ident
	{
		#[inline(always)]
		fn from_i64(x: i64) -> Option<$ident>
		{
			let min: $ident = std::num::Bounded::min_value();
			let max: $ident = std::num::Bounded::max_value();
			if x >= min as i64 && x <= max as i64
			{
				Some(unsafe {std::mem::transmute_copy(&x)})
			}
			else
			{
				None
			}
		}
		#[inline(always)]
		fn from_u64(x: u64) -> Option<$ident>
		{
			let i64_max: i64 = std::num::Bounded::max_value();
			if x > i64_max as u64
			{
				return None
			}
			std::num::FromPrimitive::from_i64(x as i64)
		}
	}
	impl std::fmt::Show for $ident
	{
		fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::FormatError>
		{
			(*self as int).fmt(formatter)
		}
	}

	ranged_type_impl_inner!($ident, Sub, sub, CheckedSub, checked_sub, checked_sub_internal)
	ranged_type_impl_inner!($ident, Add, add, CheckedAdd, checked_add, checked_add_internal)
	ranged_type_impl_inner!($ident, Div, div, CheckedDiv, checked_div, checked_div_internal)
	ranged_type_impl_inner!($ident, Mul, mul, CheckedMul, checked_mul, checked_mul_internal)
))

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
	reg.register_macro("ranged_type_enumdef", expand_syntax_ext);
}

pub fn expand_syntax_ext(cx: &mut ExtCtxt, sp: Span, tts: &[ast::TokenTree]) -> Box<base::MacResult>
{
	let (base_name, lower_expr, upper_expr) = parse_tts(cx, tts);

	let base_name_str = token::get_ident(base_name.ident);

	macro_rules! get_int( ($x:ident) => (
		match $x.node {
			// expression is a literal
			ast::ExprLit(lit) => match lit.node {
				// int literal specifically
				ast::LitInt(ref s, _) => {
					s.clone()
				}
				ast::LitIntUnsuffixed(ref s) => {
					s.clone()
				}
				_ => {
					cx.span_err($x.span, "range bounds must be literals of type int!");
					return base::DummyResult::expr(sp);
				}
			},
			_ => {
				cx.span_err($x.span, "non-literal bound given for ranged type!");
				return base::DummyResult::expr(sp);
			}
		}
	))

	let lower = get_int!(lower_expr);
	let upper = get_int!(upper_expr);

	let count: u64 = {
		if lower >= upper
		{
			cx.span_err(sp, "lower bound must be a lesser value than the upper bound!");
			return base::DummyResult::expr(sp);
		}
		else
		{
			(upper-lower) as u64
		}
	};

	let gen_ident_name=|lower: i64, index: u64| -> String
	{
		let v = lower+(index as i64);
		if v < 0
			{format!("{}Neg{}", base_name_str, std::num::abs(v))}
		else
			{format!("{}{}", base_name_str, v)}
	};

	let mut variants = vec![];

	//lower bound variant
	let first_variant = syntax::codemap::respan(sp/*TODO: some span*/,
		ast::Variant_ {
			name: cx.ident_of(gen_ident_name(lower, 0).as_slice()),
			attrs: Vec::new(),
			kind: ast::TupleVariantKind(vec![]),
			id: ast::DUMMY_NODE_ID,
			disr_expr: Some(lower_expr),
			vis: ast::Public
		}
	);

	variants.push(box(GC) first_variant);

	//add the rest of the variants
	for i in std::iter::range(1, count+1)
	{
		let ident_name = gen_ident_name(lower, i);

		let ident = cx.ident_of(ident_name.as_slice());
		variants.push(box(GC) cx.variant(sp/*TODO: some span*/, ident, vec![]));
	}

	return MacItem::new(cx.item_enum(sp, base_name.ident, ast::EnumDef {variants: variants}));
}

#[allow(dead_code)]
struct Ident
{
	ident: ast::Ident,
	span: Span
}

fn parse_tts(cx: &ExtCtxt,
	tts: &[ast::TokenTree]) -> (Ident, Gc<ast::Expr>, Gc<ast::Expr>)
{
	let mut p = cx.new_parser_from_tts(tts);
	let span_lo = p.span.lo;
	let ident = p.parse_ident();
	let span_hi = p.last_span.hi;
	let base_name = Ident {ident: ident, span: mk_sp(span_lo, span_hi)};
	p.expect(&token::COMMA);
	let lower_bound = p.parse_expr();
	p.expect(&token::COMMA);
	let upper_bound = p.parse_expr();
	if p.token != token::EOF {
		p.unexpected();
	}
	(base_name, lower_bound, upper_bound)
}
