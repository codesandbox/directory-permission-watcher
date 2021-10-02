const { Watcher } = require("./library");
const path = require("path");

class NodeWatcher {
  // Options { ignores: string[] }
  constructor(opts) {
    // The n-api watcher is a pretty basic watcher as we can call the event emitters
    // from the node-side we just need a filtered stream of events from n-api
    this.watcher = new Watcher();
    this.watcher.listen((err, ...values) => {
      console.error(err);
      console.log(values);
    });
  }

  watch(p) {
    this.watcher.watch(p);
  }

  unwatch(p) {
    this.watcher.unwatch(p);
  }

  updateIgnorePaths(paths) {
    // TODO: Implement this
  }

  dispose() {
    // TODO: Implement this
    // this.watcher.dispose();
  }
}

const w = new NodeWatcher();
w.watch(__dirname);
