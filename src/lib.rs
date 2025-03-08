#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recipe {
	tags: Vec<String>,
	ingredients: Vec<Ingredient>,
	equipment: Vec<String>,
	instructions: Vec<Instruction>,
	source: Option<Source>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ingredient {
	pub id: Option<String>,
	pub names: Vec<IngredientName>,
	pub measurements: Vec<Measurement>,
	pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IngredientName {
	pub name: String,
	pub subtype: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
	pub quantity: f32,
	pub unit: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
	pub description: String,
	pub ingredient_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Source {
	Url(url::Url),
	Other(String),
}

impl<'doc> kdlize::FromKdlNode<'doc, ()> for Recipe {
	type Error = kdlize::error::QueryError;
	fn from_kdl(node: &mut kdlize::reader::Node<'doc, ()>) -> Result<Self, Self::Error> {
		use kdlize::reader::*;
		let tags = node.children("tag").value().to::<String>().collect()??;
		let ingredients = node.children("ingredient").to().collect()?;
		let equipment = node.children("equipment").value().to::<String>().collect()??;
		let instructions = { node.child("instruction")?.children("step").to().collect()? };
		let source = node.child("source").ok().next()?.to()?;
		Ok(Self { tags, ingredients, equipment, instructions, source })
	}
}

impl kdlize::AsKdlNode for Recipe {
	fn as_kdl(&self) -> kdlize::builder::Node {
		use kdlize::builder::*;
		Node::default()
			+ Children("tag", Value(&self.tags))
			+ Children("ingredient", &self.ingredients)
			+ Children("equipment", Value(&self.equipment))
			+ Child("instructions", Node::default() + Children("step", &self.instructions))
			+ OmitIfEmpty(Child("source", Value(&self.source)))
	}
}

impl<'doc> kdlize::FromKdlNode<'doc, ()> for Ingredient {
	type Error = kdlize::error::QueryError;
	fn from_kdl(node: &mut kdlize::reader::Node<'doc, ()>) -> Result<Self, Self::Error> {
		use kdlize::reader::*;
		let id = node.prop("id").ok().to()?;
		let names = {
			let mut names = node.children("option").to::<IngredientName>().collect::<Vec<_>>()?;
			if let Ok(ingredient) = IngredientName::from_kdl(node) {
				names.push(ingredient);
			}
			names
		};
		let measurements = {
			let mut measurements = node.children("amount").to::<Measurement>().collect::<Vec<_>>()?;
			if let Ok(measurement) = Measurement::from_kdl(node) {
				measurements.push(measurement);
			}
			measurements
		};
		let notes = node.children("note").value().to().collect()??;
		Ok(Self { id, names, measurements, notes })
	}
}

impl kdlize::AsKdlNode for Ingredient {
	fn as_kdl(&self) -> kdlize::builder::Node {
		use kdlize::builder::*;
		let mut node = Node::default();

		if self.names.len() == 1 {
			node += self.names[0].as_kdl();
		} else {
			node += Children("option", &self.names);
		}

		if self.measurements.len() == 1 {
			node += self.measurements[0].as_kdl();
		} else {
			node += Children("amount", &self.measurements);
		}

		node += OmitIfEmpty(Property("id", Value(&self.id)));

		node += Children("note", Value(&self.notes));

		node
	}
}

impl<'doc> kdlize::FromKdlNode<'doc, ()> for IngredientName {
	type Error = kdlize::error::QueryError;
	fn from_kdl(node: &mut kdlize::reader::Node<'doc, ()>) -> Result<Self, Self::Error> {
		use kdlize::reader::*;
		let name = node.next()?.to()?;
		let subtype = node.prop("kind").ok().to()?;
		Ok(Self { name, subtype })
	}
}

impl kdlize::AsKdlNode for IngredientName {
	fn as_kdl(&self) -> kdlize::builder::Node {
		use kdlize::builder::*;
		Node::default() + Value(&self.name) + OmitIfEmpty(Property("kind", Value(&self.subtype)))
	}
}

impl<'doc> kdlize::FromKdlNode<'doc, ()> for Measurement {
	type Error = kdlize::error::QueryError;
	fn from_kdl(node: &mut kdlize::reader::Node<'doc, ()>) -> Result<Self, Self::Error> {
		use kdlize::reader::*;
		let quantity = {
			let entry = node.next()?;
			if let Ok(quantity) = entry.to::<i64>() { quantity as f32 } else { entry.to::<f32>()? }
		};
		let unit = node.next().ok().to()?;
		Ok(Self { quantity, unit })
	}
}

impl kdlize::AsKdlNode for Measurement {
	fn as_kdl(&self) -> kdlize::builder::Node {
		use kdlize::builder::*;
		let mut node = Node::default();

		if self.quantity == self.quantity.trunc() {
			node += Value(self.quantity.trunc() as i64);
		} else {
			node += Value(self.quantity);
		}

		node += OmitIfEmpty(Value(&self.unit));

		node
	}
}

impl<'doc> kdlize::FromKdlNode<'doc, ()> for Instruction {
	type Error = kdlize::error::QueryError;
	fn from_kdl(node: &mut kdlize::reader::Node<'doc, ()>) -> Result<Self, Self::Error> {
		use kdlize::reader::*;
		let description = node.next()?.to()?;
		let ingredient_refs = node.children("ref").value().to().collect()??;
		Ok(Self { description, ingredient_refs })
	}
}

impl kdlize::AsKdlNode for Instruction {
	fn as_kdl(&self) -> kdlize::builder::Node {
		use kdlize::builder::*;
		Node::default() + Value(&self.description) + Children("ref", Value(&self.ingredient_refs))
	}
}

kdlize::impl_kdlvalue_str!(Source);

impl std::str::FromStr for Source {
	type Err = std::convert::Infallible;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match url::Url::from_str(s) {
			Ok(url) => Ok(Self::Url(url)),
			Err(_) => Ok(Self::Other(s.to_owned())),
		}
	}
}

impl std::fmt::Display for Source {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Url(url) => write!(f, "{url}"),
			Self::Other(s) => write!(f, "{s}"),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use kdlize::AsKdlNode;

	fn parse<'doc, T>(doc: &'doc kdl::KdlDocument) -> Result<T, <T as kdlize::FromKdlNode<'doc, ()>>::Error>
	where
		T: kdlize::FromKdlNode<'doc, ()>,
	{
		let mut node = kdlize::reader::Node::new(&doc.nodes()[0], &());
		T::from_kdl(&mut node)
	}

	mod ingredient {
		use super::*;

		#[test]
		fn single_nosubtype_nounit_noid() -> anyhow::Result<()> {
			let doc_str = "ingredient Egg 1\n";
			let doc = kdl::KdlDocument::parse(doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![IngredientName { name: "Egg".into(), subtype: None }],
				id: None,
				measurements: vec![Measurement { quantity: 1.0, unit: None }],
				notes: vec![]
			});
			let kdl_str = ingredient.as_kdl().build("ingredient").to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}

		#[test]
		fn single_subtype_unit_noid() -> anyhow::Result<()> {
			let doc_str = "ingredient Sugar kind=Granulated 1 tbsp\n";
			let doc = kdl::KdlDocument::parse(doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![IngredientName { name: "Sugar".into(), subtype: Some("Granulated".into()) }],
				id: None,
				measurements: vec![Measurement { quantity: 1.0, unit: Some("tbsp".into()) }],
				notes: vec![]
			});
			let kdl_str = ingredient.as_kdl().build("ingredient").to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}

		#[test]
		fn single_subtype_unit_id() -> anyhow::Result<()> {
			let doc_str = "ingredient Sugar kind=Granulated 1 tbsp id=dry\n";
			let doc = kdl::KdlDocument::parse(doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![IngredientName { name: "Sugar".into(), subtype: Some("Granulated".into()) }],
				id: Some("dry".into()),
				measurements: vec![Measurement { quantity: 1.0, unit: Some("tbsp".into()) }],
				notes: vec![]
			});
			let kdl_str = ingredient.as_kdl().build("ingredient").to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}

		#[test]
		fn multiple_measurements() -> anyhow::Result<()> {
			use trim_margin::MarginTrimmable;
			let doc_str = "
			|ingredient Flour kind=All-Purpose id=dry {
			|    amount 2 cup
			|    amount 250 gram
			|}
			|".trim_margin().unwrap();
			let doc = kdl::KdlDocument::parse(&doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![IngredientName { name: "Flour".into(), subtype: Some("All-Purpose".into()) }],
				id: Some("dry".into()),
				measurements: vec![Measurement { quantity: 2.0, unit: Some("cup".into()) }, Measurement {
					quantity: 250.0,
					unit: Some("gram".into())
				}],
				notes: vec![]
			});
			let kdl_str = ingredient.as_kdl().build("ingredient").to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}

		#[test]
		fn multiple_measurements_options() -> anyhow::Result<()> {
			use trim_margin::MarginTrimmable;
			let doc_str = "
			|ingredient id=wet {
			|    option Milk kind=Whole
			|    option Milk kind=Buttermilk
			|    option Milk kind=\"2%\"
			|    amount 0.75 cup
			|    amount 177 ml
			|}
			|".trim_margin().unwrap();
			let doc = kdl::KdlDocument::parse(&doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![
					IngredientName { name: "Milk".into(), subtype: Some("Whole".into()) },
					IngredientName { name: "Milk".into(), subtype: Some("Buttermilk".into()) },
					IngredientName { name: "Milk".into(), subtype: Some("2%".into()) },
				],
				id: Some("wet".into()),
				measurements: vec![
					Measurement { quantity: 0.75, unit: Some("cup".into()) },
					Measurement { quantity: 177.0, unit: Some("ml".into()) },
				],
				notes: vec![]
			});
			let kdl_str = ingredient.as_kdl().build("ingredient").to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}

		#[test]
		fn notes() -> anyhow::Result<()> {
			use trim_margin::MarginTrimmable;
			let doc_str = "
			|ingredient \"Baking Powder\" 1 tbsp id=dry {
			|    note \"Some notes\"
			|}
			|".trim_margin().unwrap();
			let doc = kdl::KdlDocument::parse(&doc_str)?;
			let ingredient = parse::<Ingredient>(&doc)?;
			assert_eq!(ingredient, Ingredient {
				names: vec![IngredientName { name: "Baking Powder".into(), subtype: None }],
				id: Some("dry".into()),
				measurements: vec![Measurement { quantity: 1.0, unit: Some("tbsp".into()) }],
				notes: vec!["Some notes".into()],
			});
			let mut node = ingredient.as_kdl().build("ingredient");
			node.ensure_v2();
			let kdl_str = node.to_string();
			assert_eq!(kdl_str, doc_str);
			Ok(())
		}
	}
}
