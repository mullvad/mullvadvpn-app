import React, { Component, PropTypes } from 'react';
import { formatAccount } from '../lib/formatters';

export default class AccountInput extends Component {

  static propTypes = {
    value: PropTypes.string,
    onEnter: PropTypes.func,
    onChange: PropTypes.func
  }

  constructor(props) {
    super(props);

    this.state = { value: '', selectionRange: [0,0] };
  }

  componentWillReceiveProps(nextProps) {
    const nextVal = (nextProps.value || '').replace(/[^0-9]/g, '');
    if(nextVal !== this.state.value) {
      const len = nextVal.length;
      this.setState({ value: nextVal, selectionRange: [len, len] });
    }
  }

  shouldComponentUpdate(nextProps, nextState) {
    return (this.props.value !== nextProps.value ||
            this.props.onEnter !== nextProps.onEnter ||
            this.props.onChange !== nextProps.onChange ||
            this.state.value !== nextState.value || 
            this.state.selectionRange[0] !== nextState.selectionRange[0] || 
            this.state.selectionRange[1] !== nextState.selectionRange[1]);
  }

  insert(val, insert, selRange) {
    const head = val.slice(0, selRange[0]);
    const tail = val.slice(selRange[1], val.length);
    const newVal = head + insert + tail;
    const selectionOffset = head.length + insert.length;

    return { value: newVal, selectionRange: [selectionOffset, selectionOffset] };
  }

  remove(val, selRange) {
    let newVal, selectionOffset;

    if(selRange[0] === selRange[1]) {
      const head = val.slice(0, selRange[0] - 1);
      const tail = val.slice(selRange[0], val.length);
      newVal = head + tail;
      selectionOffset = head.length;
    } else {
      const head = val.slice(0, selRange[0]);
      const tail = val.slice(selRange[1], val.length);
      newVal = head + tail;
      selectionOffset = head.length;
    }

    return { value: newVal, selectionRange: [selectionOffset, selectionOffset] };
  }

  toInternalSelectionRange(val, domRange) {
    const countSpaces = (val) => {
      return (val.match(/\s/g) || []).length;
    };

    const fmt = formatAccount(val || '');
    let start = domRange[0];
    let end = domRange[1];
    const spacesBefore = countSpaces(fmt.slice(0, start));
    const spacesWithin = countSpaces(fmt.slice(start, end));
    const finalStart = start - spacesBefore;
    const finalEnd = end - (spacesBefore + spacesWithin);

    console.log('toInternalSelectionRange: spacesBefore: ' + spacesBefore + ', spacesWithin: ' + spacesWithin + 
      "\n<" + fmt.slice(0, start) + ">\n<" + fmt.slice(start, end) + ">" + "\nfinalStart: " + finalStart + "\nfinalEnd: " + finalEnd);
    

    return [ finalStart, finalEnd ];
  }

  toDomSelection(val, selRange) {
    const countSpaces = (val, untilIndex) => {
      if(val.length > 12) { return 0; }
      return Math.floor(untilIndex / 4); // groups of 4 digits
    };

    let start = selRange[0];
    let end = selRange[1];
    const startSpaces = countSpaces(val, start);
    const endSpaces = countSpaces(val, end);

    console.log('toDomSelection: [' + start + ', ' + end + '] startSpaces: ' + startSpaces + ', endSpaces: ' + endSpaces);
    
    start += startSpaces;
    end += startSpaces + (endSpaces - startSpaces);

    return [ start, end ];
  }

  onKeyDown(e) {
    console.log('Entered ' + e.key);

    const { value, selectionRange } = this.state;
    let result;

    // arrows.
    if(e.which >= 37 && e.which <= 40) {
      return;
    }

    // prevent native keyboard input management
    e.preventDefault();

    if(e.which === 8) { // backspace
      result = this.remove(value, selectionRange);
    } else if(/[0-9]/.test(e.key)) { // digits
      result = this.insert(value, e.key, selectionRange);
    } else { // any other keys
      return;
    }

    this.setState(result, () => {
      if(this.props.onChange) {
        this.props.onChange(result.value);
      }
    });
  }

  onSelect(e) {
    const ref = e.target;
    let start = ref.selectionStart;
    let end = ref.selectionEnd;

    console.log('onSelect: ' + start + ', ' + end);

    const { value, selectionRange: curRange } = this.state;
    const selRange = this.toInternalSelectionRange(value, [start, end]);
    // if(selRange[0] !== curRange[0] || selRange[1] !== curRange[1]) {
      this.setState({ selectionRange: selRange });
    // }
  }

  onKeyUp(e) {
    if(e.which === 13 && this.props.onEnter) {
      this.props.onEnter();
    }
    console.log('onKeyUp: ', e);
  }

  onRef(ref) {

    const { value, selectionRange } = this.state;
    const domRange = this.toDomSelection(value, selectionRange);
    // if(ref.selectionStart !== domRange[0] || ref.selectionEnd !== domRange[1]) {
      ref && ref.setSelectionRange(domRange[0], domRange[1]);
    // }

  }

  render() {
    const displayString = formatAccount(this.state.value || '');
    const props = Object.assign({}, this.props);

    // exclude built-in props
    for(let key of Object.keys(AccountInput.propTypes)) {
      if(props.hasOwnProperty(key)) {
        delete props[key];
      }
    }
    
    return (
      <input type="text" 
        value={ displayString } 
        onSelect={ ::this.onSelect }
        onKeyUp={ ::this.onKeyUp } 
        onKeyDown={ ::this.onKeyDown }
        ref={ ::this.onRef }
        { ...props } />
    );
  }

}