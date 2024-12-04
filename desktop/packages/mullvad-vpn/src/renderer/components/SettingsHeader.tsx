import * as React from 'react';
import styled from 'styled-components';

import { Colors, Spacings } from '../tokens';
import { Flex } from './common/layout';
import { LabelTiny, TitleBig } from './common/text';

export const HeaderTitle = styled(TitleBig)({
  wordWrap: 'break-word',
  hyphens: 'auto',
});

export const HeaderSubTitle = styled(LabelTiny).attrs({
  $color: Colors.white60,
})({});

interface SettingsHeaderProps {
  children?: React.ReactNode;
  className?: string;
}

function SettingsHeader(props: SettingsHeaderProps, forwardRef: React.Ref<HTMLDivElement>) {
  return (
    <Flex
      ref={forwardRef}
      $flexDirection="column"
      $gap={Spacings.spacing3}
      $margin={{
        top: Spacings.spacing3,
        horizontal: Spacings.spacing5,
        bottom: Spacings.spacing4,
      }}
      className={props.className}>
      {props.children}
    </Flex>
  );
}

export default React.forwardRef(SettingsHeader);
