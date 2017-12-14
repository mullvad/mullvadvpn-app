import { Styles } from "reactxp";

const styles = {
  settings:
  Styles.createViewStyle({
    backgroundColor: '#192E45',
    height: '100%'
  }),
  settings__container:
  Styles.createViewStyle({
    flexDirection: "column",
    height: '100%'
  }),
  settings__header:
  Styles.createViewStyle({
    flex: 0,
    paddingTop: 12,
    paddingBottom: 12,
    paddingLeft: 24,
    paddingRight: 24,
    position: "relative" /* anchor for close button */
  }),
  settings__content:
  Styles.createViewStyle({
    flexDirection: "column",
    flex: 1,
  }),
  settings__close:
  Styles.createViewStyle({
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "flex-start",
    marginTop: 0,
    marginLeft: 12,
  }),
  settings__close_icon:
  Styles.createViewStyle({
    width: 24,
    height: 24,
    flex: 0,
    opacity: 0.6,
    marginRight: 8,
  }),
  settings__title:
  Styles.createTextStyle({
    fontFamily: "DINPro",
    fontSize: 32,
    fontWeight: "900",
    lineHeight: 40,
    color: "#FFFFFF"
  }),
  settings__cell:
  Styles.createViewStyle({
    backgroundColor: "rgba(41,71,115,1)",
    paddingTop: 15,
    paddingBottom: 15,
    paddingLeft: 24,
    paddingRight: 24,
    marginLeft: -6, //Because of button.css, when removed remove this
    marginRight: -6, //Because of button.css, when removed remove this
    marginTop: -1, //Because of button.css, when removed remove this
    marginBottom: 0, //Because of button.css, when removed remove this
    flex: 1,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between"
  }),
  settings__cell_disclosure:
  Styles.createViewStyle({
    marginLeft: 8
  }),
  settings__cell_spacer:
  Styles.createViewStyle({
    height: 24,
    flex: 0
  }),
  settings__cell__active_hover:
  Styles.createViewStyle({
    backgroundColor: "rgba(41,71,115,0.9)"
  }),
  settings__cell_label:
  Styles.createTextStyle({
    fontFamily: "DINPro",
    fontSize: 20,
    fontWeight: "900",
    lineHeight: 26,
    color: "#FFFFFF"
  }),
  settings__cell_icon:
  Styles.createViewStyle({
    width: 16,
    height: 16,
    flex: 0,
    marginRight: 8,
    opacity: 0.8
  }),
  settings__account_paid_until_label:
  Styles.createTextStyle({
    fontFamily: "Open Sans",
    fontSize: 13,
    fontWeight: "800",
    color: "rgba(255, 255, 255, 0.8)"
  }),
  settings__account_paid_until_label__error:
  Styles.createViewStyle({
    color: "#d0021b"
  }),
  settings__footer_button_label:
  Styles.createTextStyle({
    fontFamily: "DINPro",
    fontSize: 20,
    fontWeight: "900",
    lineHeight: 26,
    color: "rgba(255,255,255,0.8)"
  }),
  settings__footer_button:
  Styles.createViewStyle({
    backgroundColor: "rgba(208,2,27,1)",
    paddingTop: 7,
    paddingLeft: 12,
    paddingRight: 12,
    paddingBottom: 9,
    borderRadius: 4,
    justifyContent: "center",
    alignItems: "center",
    width: '100%',
  }),
  settings__footer:
  Styles.createViewStyle({
    width: '100%',
    justifyContent: "center",
    alignItems: "center",
    paddingTop: 24,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 24,
  })
};

module.exports = styles;
