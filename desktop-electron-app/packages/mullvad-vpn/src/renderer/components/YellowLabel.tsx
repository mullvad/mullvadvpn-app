import styled from 'styled-components';

import { colors } from '../../config.json';

export default styled.span({
  display: 'inline-block',
  fontFamily: 'Open Sans',
  color: colors.blue,
  fontSize: '12px',
  fontWeight: 800,
  lineHeight: '20px',
  padding: '1px 8px',
  marginLeft: '8px',
  background: colors.yellow,
  borderRadius: '5px',
  textAlign: 'center',
  verticalAlign: 'middle',
});
