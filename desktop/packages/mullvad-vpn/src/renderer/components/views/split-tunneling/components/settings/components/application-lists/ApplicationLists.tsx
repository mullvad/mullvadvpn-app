import { Flex } from '../../../../../../../lib/components';
import { Accordion } from '../../../../../../../lib/components/accordion';
import { useShowNonSplitApplicationList, useShowSplitApplicationList } from '../../hooks';
import { NonSplitApplicationSection, SplitApplicationSection } from './components';

export function ApplicationLists() {
  // TODO: The parent hooks should be renamed to something more generic if they
  // are to be reused in this component.
  const showNonSplitApplicationSection = useShowNonSplitApplicationList();
  const showSplitApplicationSection = useShowSplitApplicationList();

  return (
    <Flex $flexDirection="column" $gap="medium">
      <Accordion expanded={showSplitApplicationSection}>
        <SplitApplicationSection />
      </Accordion>
      <Accordion expanded={showNonSplitApplicationSection}>
        <NonSplitApplicationSection />
      </Accordion>
    </Flex>
  );
}
