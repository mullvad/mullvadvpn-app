import styled from 'styled-components';

import { durations } from '../../../lib/foundations';
import Accordion from '../../Accordion';

export const ConnectionPanelAccordion = styled(Accordion)({
  transition: `height ${durations.normal} ease-out`,
});
