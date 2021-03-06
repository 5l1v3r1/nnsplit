import * as nnsplit from "nnsplit";

async function test() {
  let splitter = await new nnsplit.NNSplit("/de/model.onnx");

  let splits = await splitter.split([
    "Das ist ein Test Das ist noch ein Test.",
  ]);
  splits = splits[0];
  let isExpected =
    splits.parts[0].text == "Das ist ein Test " &&
    splits.parts[1].text == "Das ist noch ein Test.";
  if (!isExpected) {
    console.error("split in unexpected parts.");
  }

  console.log("cypress:success");
}

async function benchmark() {
  let splitter = await tfl.loadLayersModel("/de/model.onnx");

  let input = tf.tensor(new Uint8Array(100), [1, 100]);
  console.time("backend");

  splitter.predict(input);
  console.timeEnd("backend");

  // let splitter = await new nnsplit.NNSplit("/de/model.json");

  // fetch("sample.json").then((response) => response.json()).then((data) => {
  //     console.time("split");

  //     splitter.split([data[0], data[1]]);
  //     console.timeEnd("split");

  //     console.time("split");

  //     splitter.split([data[0], data[1]]);
  //     console.timeEnd("split");

  //     console.time("split");

  //     splitter.split([data[0]]);
  //     console.timeEnd("split");

  //     console.time("split");

  //     splitter.split([data[0]]);
  //     console.timeEnd("split");
  // });
}

export { test, benchmark };
