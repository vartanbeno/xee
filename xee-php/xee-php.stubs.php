<?php

// Stubs for xee-php

namespace Xee {
    /**
     * Documents hold XML documents that can be queried.
     */
    class Documents {
        public static function makeNew(): \Xee\Documents {}

        /**
         * Add a document to the Documents store from a string.
         *
         * The string must be well-formed XML.
         */
        public function addString(string $uri, string $content): \Xee\DocumentHandle {}
    }

    /**
     * An iterator over a sequence
     */
    class SequenceIterator implements ce :: iterator() {
        /**
         * Rewind the iterator to the start.
         */
        public function rewind() {}

        /**
         * Get the current item in the sequence.
         */
        public function current(): mixed {}

        /**
         * Get the key of the current item in the sequence.
         *
         * This is the position in the sequence.
         */
        public function key(): mixed {}

        /**
         * Move to the next item in the sequence.
         */
        public function next() {}

        /**
         * Check if the current position is valid.
         */
        public function valid(): bool {}
    }

    /**
     * A sequence of items returned by an XPath query.
     *
     * This can be treated as an array and you can iterate over it.
     */
    class Sequence implements ce :: arrayaccess(), ce :: countable(), ce :: aggregate() {
        public function count(): int {}

        public function offsetExists(mixed $offset): bool {}

        public function offsetGet(mixed $offset): mixed {}

        public function offsetSet(mixed $_offset, mixed $_value): mixed {}

        public function offsetUnset(mixed $_offset): mixed {}

        public function getIterator(): \Xee\SequenceIterator {}
    }

    /**
     * A handle to a document in the documents store.
     *
     * You can use it to perform a query.
     */
    class DocumentHandle {
    }

    /**
     * A collection of XPath queries that can be executed against a document.
     *
     * You can compile XPath expressions into queries using this store.
     */
    class Queries {
        public static function makeNew(): \Xee\Queries {}

        /**
         * A sequence query returns an XPath sequence.
         *
         * The query must be a valid XPath 3.1 expression.
         */
        public function sequence(string $query): \Xee\SequenceQuery {}
    }

    /**
     * A compiled XPath query that returns a sequence.
     */
    class SequenceQuery {
        /**
         * Execute the query against a session and a document handle.
         */
        public function execute(\Xee\Documents $documents, \Xee\DocumentHandle $doc): \Xee\Sequence {}
    }
}
