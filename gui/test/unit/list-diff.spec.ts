import { expect } from 'chai';
import { it, describe } from 'mocha';
import { calculateItemList, RowDisplayData } from '../../src/renderer/components/List';

const prevItems: Array<RowDisplayData<undefined>> = [
  { key: 'a', data: undefined, removing: false },
  { key: 'b', data: undefined, removing: false },
  { key: 'c', data: undefined, removing: false },
  { key: 'd', data: undefined, removing: false },
];

describe('List diff', () => {
  it('Should add item to the beginning', () => {
    const nextItems: Array<RowDisplayData<undefined>> = [
      { key: '1', data: undefined, removing: false },
      ...prevItems,
    ];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(5);
    expect(combinedItems.slice(1)).to.deep.equal(prevItems);
    expect(combinedItems).to.deep.equal(nextItems);
  });

  it('Should add item to the end', () => {
    const nextItems: Array<RowDisplayData<undefined>> = [
      ...prevItems,
      { key: '1', data: undefined, removing: false },
    ];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(5);
    expect(combinedItems.slice(0, 4)).to.deep.equal(prevItems);
    expect(combinedItems).to.deep.equal(nextItems);
  });

  it('Should add item to the middle', () => {
    const nextItems: Array<RowDisplayData<undefined>> = [
      ...prevItems.slice(0, 2),
      { key: '1', data: undefined, removing: false },
      ...prevItems.slice(2),
    ];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(5);
    expect([...combinedItems.slice(0, 2), ...combinedItems.slice(3)]).to.deep.equal(prevItems);
    expect(combinedItems).to.deep.equal(nextItems);
  });

  it('Should remove first item', () => {
    const nextItems = prevItems.slice(1);
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(4);
    expect(combinedItems.slice(1, 4)).to.deep.equal(prevItems.slice(1, 4));
    expect(combinedItems[0]).to.deep.equal({ ...prevItems[0], removing: true });
  });

  it('Should remove last item', () => {
    const nextItems = prevItems.slice(0, -1);
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(4);
    expect(combinedItems.slice(0, -1)).to.deep.equal(prevItems.slice(0, -1));
    expect(combinedItems.at(-1)).to.deep.equal({ ...prevItems.at(-1), removing: true });
  });

  it('Should remove middle item', () => {
    const nextItems = [...prevItems.slice(0, 1), ...prevItems.slice(2)];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(4);
    expect(combinedItems.slice(0, 1)).to.deep.equal(prevItems.slice(0, 1));
    expect(combinedItems.slice(2)).to.deep.equal(prevItems.slice(2));
    expect(combinedItems[1]).to.deep.equal({ ...prevItems[1], removing: true });
  });

  it('should both add and remove items', () => {
    const nextItems = [
      { key: '1', data: undefined, removing: false },
      ...prevItems.slice(1, -1),
      { key: '2', data: undefined, removing: false },
    ];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.have.length(6);
    expect(combinedItems[0]).to.deep.equal({ ...prevItems[0], removing: true });
    expect(combinedItems[1]).to.deep.equal(nextItems[0]);
    expect(combinedItems.slice(2, -2)).to.deep.equal(prevItems.slice(1, -1));
    expect(combinedItems.at(-2)).to.deep.equal({ ...prevItems.at(-1), removing: true });
    expect(combinedItems.at(-1)).to.deep.equal(nextItems.at(-1));
  });

  it('should remove item being removed', () => {
    const prevItems: Array<RowDisplayData<undefined>> = [
      { key: '1', data: undefined, removing: true },
    ];
    const nextItems: Array<RowDisplayData<undefined>> = [];
    const combinedItems = calculateItemList(prevItems, nextItems);

    expect(combinedItems).to.deep.equal(prevItems);
  });
});
