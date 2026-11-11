import * as Y from 'yjs';
import { LOCAL, type BoardDoc } from './doc';

/** An undo history over the board's shapes that records only this client's `LOCAL` writes, never a peer's. */
export function createUndo(doc: BoardDoc): Y.UndoManager {
	return new Y.UndoManager(doc.shapes, { trackedOrigins: new Set([LOCAL]), captureTimeout: 400 });
}
