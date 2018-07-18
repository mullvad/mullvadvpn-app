// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  select_location: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  title_header: Styles.createViewStyle({
    paddingBottom: 0,
  }),
  subtitle_header: Styles.createViewStyle({
    paddingTop: 0,
  }),
  content: Styles.createViewStyle({
    overflow: 'visible',
  }),
  relay_status: Styles.createViewStyle({
    width: 16,
    height: 16,
    borderRadius: 8,
    marginLeft: 4,
    marginRight: 4,
    marginTop: 20,
    marginBottom: 20,
  }),
  relay_status__inactive: Styles.createViewStyle({
    backgroundColor: colors.red95,
  }),
  relay_status__active: Styles.createViewStyle({
    backgroundColor: colors.green90,
  }),
  tick_icon: Styles.createViewStyle({
    color: colors.white,
    marginLeft: 0,
    marginRight: 0,
    marginTop: 15.5,
    marginBottom: 15.5,
  }),
  country: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 0,
  }),
  collapse_button: Styles.createViewStyle({
    flex: 0,
    alignSelf: 'stretch',
    justifyContent: 'center',
    paddingRight: 16,
    paddingLeft: 16,
  }),
  cell: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 20,
    paddingRight: 0,
  }),
  sub_cell: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 40,
    backgroundColor: colors.blue40,
  }),
  sub_cell__selected: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 40,
    backgroundColor: colors.green,
  }),
  cell_selected: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 20,
    paddingRight: 0,
    backgroundColor: colors.green,
  }),
  expand_chevron_hover: Styles.createViewStyle({
    color: colors.white,
  }),
};
