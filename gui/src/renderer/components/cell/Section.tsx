import React, { useEffect } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { useAppContext } from '../../context';
import { useHistory } from '../../lib/history';
import { useBoolean } from '../../lib/utilityHooks';
import Accordion from '../Accordion';
import ChevronButton from '../ChevronButton';
import { buttonText, openSans, sourceSansPro } from '../common-styles';
import { Container } from './Container';
import { Row } from './Row';

const StyledSection = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

interface SectionTitleProps {
  disabled?: boolean;
  thin?: boolean;
}

export const SectionTitle = styled(Row)(buttonText, (props: SectionTitleProps) => ({
  paddingRight: '16px',
  color: props.disabled ? colors.white20 : colors.white,
  fontWeight: props.thin ? 400 : 600,
  fontSize: props.thin ? '15px' : '18px',
  ...(props.thin ? openSans : sourceSansPro),
}));

export const CellSectionContext = React.createContext<boolean>(false);

interface SectionProps extends React.HTMLAttributes<HTMLDivElement> {
  sectionTitle?: React.ReactElement;
}

export function Section(props: SectionProps) {
  const { children, sectionTitle, ...otherProps } = props;
  return (
    <StyledSection {...otherProps}>
      <CellSectionContext.Provider value={true}>
        {sectionTitle && <StyledTitleContainer>{sectionTitle}</StyledTitleContainer>}
        {children}
      </CellSectionContext.Provider>
    </StyledSection>
  );
}

const StyledChevronButton = styled(ChevronButton)({
  padding: 0,
  marginRight: '16px',
});

const StyledTitleContainer = styled(Container)({
  display: 'flex',
  padding: 0,
});

interface ExpandableSectionProps extends SectionProps {
  expandableId: string;
  expandedInitially?: boolean;
}

export function ExpandableSection(props: ExpandableSectionProps) {
  const { expandableId, expandedInitially, sectionTitle, ...otherProps } = props;

  const history = useHistory();
  const { setNavigationHistory } = useAppContext();
  const expandedValue =
    history.location.state.expandedSections[props.expandableId] ?? !!expandedInitially;
  const [expanded, , , toggleExpanded] = useBoolean(expandedValue);

  useEffect(() => {
    history.location.state.expandedSections[props.expandableId] = expanded;
    setNavigationHistory(history.asObject);
  }, [expanded]);

  const title = (
    <>
      {sectionTitle}
      <StyledChevronButton up={expanded} onClick={toggleExpanded} />
    </>
  );

  return (
    <Section className={props.className} sectionTitle={title} {...otherProps}>
      <Accordion expanded={expanded}>{props.children}</Accordion>
    </Section>
  );
}
