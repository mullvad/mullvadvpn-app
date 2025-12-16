import { Flex } from '../../../../../../../lib/components';
import Accordion from '../../../../../../Accordion';
import { useHasNonSplitApplications, useHasSplitApplications } from '../../hooks';
import { NonSplitApplicationSection, SplitApplicationSection } from './components';

export function ApplicationLists() {
  const hasNonSplitApplications = useHasNonSplitApplications();
  const hasSplitApplications = useHasSplitApplications();

  return (
    <Flex flexDirection="column" gap="medium">
      <Accordion expanded={hasSplitApplications}>
        <SplitApplicationSection />
      </Accordion>
      <Accordion expanded={hasNonSplitApplications}>
        <NonSplitApplicationSection />
      </Accordion>
    </Flex>
  );
}
