export const formatAccount = (val) => {
    // display number altogether when longer than 12
    if(val.length > 12) {
      return val;
    }

    // display quartets
    return val.replace(/([0-9]{4})/g, '$1 ').trim();
};
