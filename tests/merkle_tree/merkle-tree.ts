import { sha256 } from "js-sha256";
import invariant from "tiny-invariant";
import fs from "fs";

function getPairElement(idx: number, layer: Buffer[]): Buffer | null {
  const pairIdx = idx % 2 === 0 ? idx + 1 : idx - 1;

  if (pairIdx < layer.length) {
    const pairEl = layer[pairIdx];
    invariant(pairEl, "pairEl");
    return pairEl;
  } else {
    return null;
  }
}

function bufDedup(elements: Buffer[]): Buffer[] {
  return elements.filter((el, idx) => {
    return idx === 0 || !elements[idx - 1]?.equals(el);
  });
}

function bufArrToHexArr(arr: Buffer[]): string[] {
  if (arr.some((el) => !Buffer.isBuffer(el))) {
    throw new Error("Array is not an array of buffers");
  }

  return arr.map((el) => "0x" + el.toString("hex"));
}

function sortAndConcat(...args: Buffer[]): Buffer {
  return Buffer.concat([
    Buffer.from([1]),
    Buffer.concat([...args].sort(Buffer.compare.bind(null))),
  ]);
}

export class MerkleTree {
  private readonly _elements: Buffer[];
  private readonly _bufferElementPositionIndex: {
    [hexElement: string]: number;
  };
  private readonly _layers: Buffer[][];

  constructor(elements: Buffer[]) {
    this._elements = [...elements];
    // Sort elements
    this._elements.sort(Buffer.compare.bind(null));
    // Deduplicate elements
    this._elements = bufDedup(this._elements);

    this._bufferElementPositionIndex = this._elements.reduce<{
      [hexElement: string]: number;
    }>((memo, el, index) => {
      memo[el.toString("hex")] = index;
      return memo;
    }, {});

    // Create layers
    this._layers = this.getLayers(this._elements);
  }

  getLayers(elements: Buffer[]): Buffer[][] {
    if (elements.length === 0) {
      throw new Error("empty tree");
    }

    const layers = [];
    layers.push(elements);

    // Get next layer until we reach the root
    while ((layers[layers.length - 1]?.length ?? 0) > 1) {
      const nextLayerIndex: Buffer[] | undefined = layers[layers.length - 1];
      invariant(nextLayerIndex, "nextLayerIndex");
      layers.push(this.getNextLayer(nextLayerIndex));
    }

    return layers;
  }

  getNextLayer(elements: Buffer[]): Buffer[] {
    return elements.reduce<Buffer[]>((layer, el, idx, arr) => {
      if (idx % 2 === 0) {
        // Hash the current element with its pair element
        const pairEl = arr[idx + 1];
        layer.push(MerkleTree.combinedHash(el, pairEl));
      }

      return layer;
    }, []);
  }

  static combinedHash(first: Buffer, second: Buffer | undefined): Buffer {
    if (!first) {
      invariant(second, "second element of pair must exist");
      return second;
    }
    if (!second) {
      invariant(first, "first element of pair must exist");
      return first;
    }

    return Buffer.from(sha256(sortAndConcat(first, second)), "hex");
  }

  getPartialTree(depth: number): Buffer[][] {
    const newLayer = [...this._layers];
    return newLayer.reverse().slice(1, depth);
  }

  getPartialBfsTree(depth: number): Buffer[] {
    if (depth < 1) {
      throw new Error("Depth must be at least 1");
    }

    const nodes: Buffer[] = [];
    const queue: { node: Buffer; layer: number }[] = [];

    // Start with the root node
    const root = this.getRoot();
    queue.push({ node: root, layer: this._layers.length - 1 });

    while (queue.length > 0) {
      const { node, layer } = queue.shift()!;

      // Add the current node to the nodes array
      nodes.push(node);

      // If we've reached the desired depth, don't enqueue children
      if (this._layers.length - layer >= depth) {
        continue;
      }

      // Calculate the index of the current node in its layer
      const currentLayer = this._layers[layer];
      const nodeIndex = currentLayer.findIndex((el) => el.equals(node));

      if (nodeIndex === -1) {
        throw new Error("Node not found in its layer");
      }

      // Calculate indices of child nodes in the layer below
      const childLayer = layer - 1;
      if (childLayer < 0) {
        // Reached the leaves
        continue;
      }

      const leftChildIdx = nodeIndex * 2;
      const rightChildIdx = leftChildIdx + 1;

      const childLayerElements = this._layers[childLayer];

      if (leftChildIdx < childLayerElements.length) {
        queue.push({
          node: childLayerElements[leftChildIdx],
          layer: childLayer,
        });
      }

      if (rightChildIdx < childLayerElements.length) {
        queue.push({
          node: childLayerElements[rightChildIdx],
          layer: childLayer,
        });
      }
    }

    // Dont use root
    nodes.shift();
    return nodes;
  }

  getRoot(): Buffer {
    const root = this._layers[this._layers.length - 1]?.[0];
    invariant(root, "root");
    return root;
  }

  getHexRoot(): string {
    return this.getRoot().toString("hex");
  }

  getProof(el: Buffer): Buffer[] {
    const initialIdx = this._bufferElementPositionIndex[el.toString("hex")];
    if (typeof initialIdx !== "number") {
      throw new Error("Element does not exist in Merkle tree");
    }

    let idx = initialIdx;
    return this._layers.reduce((proof, layer) => {
      const pairElement = getPairElement(idx, layer);
      if (pairElement) {
        proof.push(pairElement);
      }

      idx = Math.floor(idx / 2);
      return proof;
    }, []);
  }

  getPartialProof(
    el: Buffer,
    depth: number
  ): { proof: Buffer[]; index: number } {
    const initialIdx = this._bufferElementPositionIndex[el.toString("hex")];

    if (typeof initialIdx !== "number") {
      throw new Error("Element does not exist in Merkle tree");
    }

    let idx = initialIdx;
    const proof = this._layers
      .slice(0, this._layers.length - depth)
      .reduce((proof, layer) => {
        const pairElement = getPairElement(idx, layer);
        if (pairElement) {
          proof.push(pairElement);
        }

        idx = Math.floor(idx / 2);
        return proof;
      }, []);

    return { proof, index: initialIdx };
  }

  getHexProof(el: Buffer): string[] {
    const proof = this.getProof(el);

    return bufArrToHexArr(proof);
  }
}