use xee_name::Namespaces;

use crate::{
    catalog::Catalog, environment::XPathEnvironmentSpec, error::Result, testcase::TestCase,
    testset::TestSet,
};

#[derive(Debug)]
pub(crate) struct XPathTestCase {
    test_case: TestCase<XPathEnvironmentSpec>,
}

impl XPathTestCase {
    fn namespaces<'a>(
        &'a self,
        catalog: &'a Catalog<XPathEnvironmentSpec>,
        test_set: &'a TestSet<XPathEnvironmentSpec>,
    ) -> Result<Namespaces<'a>> {
        let environments = self
            .test_case
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let mut namespaces = Namespaces::default();
        for environment in environments {
            namespaces.add(&environment.namespace_pairs())
        }
        Ok(namespaces)
    }
}
