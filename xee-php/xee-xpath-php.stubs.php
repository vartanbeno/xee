<?php

// Stubs for xee-xpath-php

namespace {
    function hello_world(string $name): string {}

    class DocumentHandle {
    }

    class SequenceQuery {
        public function execute(\Session $session, \DocumentHandle $doc): \Sequence {}
    }

    class Documents {
        public static function makeNew(): \Documents {}

        public function addString(string $uri, string $content): \DocumentHandle {}

        public function session(): \Session {}
    }

    class Queries {
        public static function makeNew(): \Queries {}

        public function sequence(string $query): \SequenceQuery {}
    }

    class Sequence {
        public function len(): int {}
    }

    class Session {
    }
}
