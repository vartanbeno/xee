<?php

// Stubs for xee-php

namespace Xee {
    class Item {
    }

    class DocumentHandle {
    }

    class Session {
    }

    class Documents {
        public static function makeNew(): \Xee\Documents {}

        public function addString(string $uri, string $content): \Xee\DocumentHandle {}

        public function session(): \Xee\Session {}
    }

    class SequenceQuery {
        public function execute(\Xee\Session $session, \Xee\DocumentHandle $doc): \Xee\Sequence {}
    }

    class Sequence implements ce :: arrayaccess(), ce :: countable() {
        public function count(): int {}

        public function offsetExists(mixed $offset): bool {}

        public function offsetGet(mixed $offset): \Xee\Item {}

        public function offsetSet(mixed $_offset, mixed $_value): mixed {}

        public function offsetUnset(mixed $_offset): mixed {}
    }

    class Queries {
        public static function makeNew(): \Xee\Queries {}

        public function sequence(string $query): \Xee\SequenceQuery {}
    }
}
