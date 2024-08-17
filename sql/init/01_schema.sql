--
-- PostgreSQL database dump
--

-- Dumped from database version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: counter; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.counter (
    user_id text NOT NULL,
    emote character(2) NOT NULL,
    count integer DEFAULT 0 NOT NULL,
    CONSTRAINT counter_count_check CHECK ((count > 0))
);


--
-- Name: options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.options (
    user_id text NOT NULL,
    opt_out boolean DEFAULT false NOT NULL,
    silent boolean DEFAULT false NOT NULL
);


--
-- Name: counter counter_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.counter
    ADD CONSTRAINT counter_pkey PRIMARY KEY (user_id, emote);


--
-- Name: options options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.options
    ADD CONSTRAINT options_pkey PRIMARY KEY (user_id);


--
-- PostgreSQL database dump complete
--

