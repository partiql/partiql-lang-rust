// Copyright Amazon.com, Inc. or its affiliates.

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "peg/partiql.pest"]
pub(crate) struct PartiQLParser;
